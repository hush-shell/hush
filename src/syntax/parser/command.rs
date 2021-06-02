use crate::io::{self, FileDescriptor};
use super::{
	ast,
	ArgPart,
	ArgUnit,
	CommandOperator as Operator,
	Error,
	ErrorReporter,
	Parser,
	Token,
	TokenKind,
};


impl<I, E> Parser<I, E>
where
	I: Iterator<Item = Token>,
	E: ErrorReporter,
{
	pub(super) fn parse_command_block(&mut self) -> Result<Box<[ast::Command]>, Error> {
		let commands = self.semicolon_sep(Self::parse_command)?;
		self.expect(TokenKind::CloseCommand)?;
		Ok(commands)
	}


	fn parse_command(&mut self) -> Result<ast::Command, Error> {
		let mut basic_commands = Vec::with_capacity(1); // Expect at least one command.

		let first = self.parse_basic_command()?;

		basic_commands.push(first);

		// Contrary to semicolons and commas, there may be no trailing pipe.
		while let Some(Token { token: TokenKind::Pipe, .. }) = self.token {
			self.step();

			let basic_command = self.parse_basic_command()?;

			basic_commands.push(basic_command);
		}

		Ok(basic_commands.into_boxed_slice().into())
	}


	fn parse_basic_command(&mut self) -> Result<ast::BasicCommand, Error> {
		let command = self.parse_argument()?;
		let pos = command.pos;

		let mut arguments = Vec::new();
		loop {
			let is_redirection = matches!(
				&self.token,
				// The current token is a single unquoted number
				Some(Token { token: TokenKind::Argument(parts), .. })
					if matches!(parts.as_ref(), &[ref part] if part.is_unquoted_number())
					// And the next token is a redirection operator.
					&& matches!(
						self.cursor.peek(),
						Some(Token { token: TokenKind::CmdOperator(op), .. })
							if op.is_redirection()
					)
			);

			if is_redirection {
				break;
			}

			if let Ok(arg) = self.parse_argument() {
				arguments.push(arg);
			} else {
				break;
			}
		}

		let (redirections, abort_on_error) = self.parse_operators()?;

		// Make sure there are no trailing arguments.
		match &self.token {
			Some(Token { token, .. }) if token.is_basic_command_end() => {
				Ok(
					ast::BasicCommand {
						command,
						arguments: arguments.into(),
						redirections: redirections.into(),
						abort_on_error,
						pos,
					}
				)
			}

			Some(token) => Err(Error::unexpected_msg(token.clone(), "end of command")),

			None => Err(Error::unexpected_eof())
		}
	}


	fn parse_argument(&mut self) -> Result<ast::Argument, Error> {
		let (token_parts, pos) = self.eat(|token| match token {
			Token { token: TokenKind::Argument(parts), pos } => Ok((parts, pos)),
			token => Err((Error::unexpected_msg(token.clone(), "argument"), token)),
		})?;

		let mut parts = Vec::<ast::ArgPart>::new();
		let mut literal = Vec::<u8>::new();

		let join_literal = |literal: &mut Vec<u8>, lit: Box<[u8]>| {
			if literal.is_empty() {
				*literal = lit.into(); // Reuse allocation.
			} else {
				literal.extend(lit.iter())
			}
		};

		let push_literal = |literal: &mut Vec<u8>, parts: &mut Vec<ast::ArgPart>| {
			if !literal.is_empty() {
				let literal = std::mem::take(literal).into();
				parts.push(
					ast::ArgPart::Unit(ast::ArgUnit::Literal(literal))
				);
			}
		};

		let push_dollar = |literal: &mut Vec<u8>, parts: &mut Vec<ast::ArgPart>, id: ast::Symbol| {
			push_literal(literal, parts);
			parts.push(
				ast::ArgPart::Unit(ast::ArgUnit::Dollar(id))
			);
		};

		for part in token_parts.into_vec() { // Use vec's owned iterator.
			match part {
				ArgPart::SingleQuoted(lit) => join_literal(&mut literal, lit),

				ArgPart::DoubleQuoted(units) => for unit in units.into_vec() {
					match unit {
						ArgUnit::Dollar(identifier) => push_dollar(&mut literal, &mut parts, identifier),
						// Literals in double quotes don't expand to patterns.
						ArgUnit::Literal(lit) => join_literal(&mut literal, lit),
					}
				}

				ArgPart::Unquoted(unit) => {
					match unit {
						ArgUnit::Dollar(identifier) => push_dollar(&mut literal, &mut parts, identifier),
						// TODO: parse patterns (home, range, collection, star, question, charclass)
						ArgUnit::Literal(lit) => join_literal(&mut literal, lit),
					}
				}
			}
		}

		// Push the trailing literal, if any.
		push_literal(&mut literal, &mut parts);

		Ok(
			ast::Argument {
				parts: parts.into(),
				pos
			}
		)
	}


	fn parse_operators(&mut self) -> Result<(Box<[ast::Redirection]>, bool), Error> {
		let mut redirections = Vec::new();

		loop {
			match &self.token {
				Some(Token { token, .. }) if token.is_basic_command_end() => break,

				Some(Token { token: TokenKind::CmdOperator(Operator::Try), .. }) => {
					self.step();

					return Ok((redirections.into(), true));
				}

				Some(_) => {
					let redirection = self.parse_redirection()?;

					redirections.push(redirection);
				}

				None => return Err(Error::unexpected_eof()),
			}
		}

		Ok((redirections.into(), false))
	}


	fn parse_redirection(&mut self) -> Result<ast::Redirection, Error> {
		match &self.token {
			&Some(Token { token: TokenKind::CmdOperator(Operator::Input { literal }), .. }) => {
				self.step();

				let source = self.parse_argument()?;

				Ok(
					ast::Redirection::Input { literal, source }
				)
			}

			Some(_) => {
				let source_fd = self
					.parse_file_descriptor()
					.unwrap_or(io::stdout_fd());
				let redirection = self.parse_output_redirection(source_fd)?;

				Ok(redirection)
			}

			None => Err(Error::unexpected_eof()),
		}
	}


	fn parse_output_redirection(&mut self, source: FileDescriptor) -> Result<ast::Redirection, Error> {
		match &self.token {
			&Some(Token { token: TokenKind::CmdOperator(Operator::Output { append }), .. }) => {
				self.step();

				let target = if append { // >> file
					let target = self.parse_argument()?;
					ast::RedirectionTarget::Append(target)
				} else if let Some(fd) = self.parse_file_descriptor() { // > fd
					ast::RedirectionTarget::Fd(fd)
				} else { // > file
					let target = self.parse_argument()?;
					ast::RedirectionTarget::Overwrite(target)
				};

				Ok(
					ast::Redirection::Output { source, target }
				)
			}

			Some(token) => Err(Error::unexpected_msg(token.clone(), "output redirection")),

			None => Err(Error::unexpected_eof()),
		}
	}


	/// Parse a optional file descriptor from a argument.
	fn parse_file_descriptor(&mut self) -> Option<FileDescriptor> {
		match &self.token {
			Some(Token { token: TokenKind::Argument(parts), .. }) => {
				match parts.as_ref() {
					&[ArgPart::Unquoted(ArgUnit::Literal(ref lit))] => {
						let lit = std::str::from_utf8(&lit).ok()?;
						let number: u8 = lit.parse().ok()?;

						self.step();

						Some(number.into())
					},

					_ => None,
				}
			}

			_ => None,
		}
	}


	/// Semicolon-separated items.
	fn semicolon_sep<P, R>(&mut self, parse: P) -> Result<Box<[R]>, Error>
	where
		P: FnMut(&mut Self) -> Result<R, Error>,
	{
		self.sep_by(parse, |token| *token == TokenKind::Semicolon)
	}
}
