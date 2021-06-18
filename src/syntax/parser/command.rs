use crate::io::{self, FileDescriptor};
use super::{
	ast,
	sync::{self, WithSync, ResultExt},
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
	/// Parse a command block.
	pub(super) fn parse_command_block(&mut self) -> sync::Result<ast::CommandBlock, Error> {
		let kind = self
			.eat(
				|token| ast::CommandBlockKind
					::from_token(&token.kind)
					.ok_or_else(
						|| (Error::unexpected_msg(token.clone(), "command block"), token)
					)
			)
			.with_sync(sync::Strategy::skip_one())?;

		// Check empty command block.
		match &self.token {
			Some(Token { kind: TokenKind::CloseCommand, pos }) => {
				return Err(Error::empty_command_block(*pos))
					.with_sync(sync::Strategy::skip_one())?;
			}
			_ => (),
		}

		let head = self.parse_command();

		let tail = match &self.token {
			Some(Token { kind: TokenKind::Semicolon, .. }) => {
				self.step();

				self.semicolon_sep(
					|parser| Ok(parser.parse_command()),
					|token| *token == TokenKind::CloseCommand,
				)
			},

			_ => Default::default(),
		};

		self.expect(TokenKind::CloseCommand)
			.with_sync(sync::Strategy::token(TokenKind::CloseCommand))?;

		Ok(ast::CommandBlock { kind, head, tail })
	}


	/// Parse a complete command, including pipelines.
	fn parse_command(&mut self) -> ast::Command {
		let mut tail = Vec::new();

		let head = self.parse_basic_command()
			.synchronize(self);

		// Contrary to semicolons and commas, there may be no trailing pipe.
		while let Some(Token { kind: TokenKind::Pipe, .. }) = self.token {
			self.step();

			let basic_command = self.parse_basic_command()
				.synchronize(self);

			tail.push(basic_command);
		}

		ast::Command {
			head,
			tail: tail.into(),
		}
	}


	/// Parse a single basic command, including redirections and try operator.
	fn parse_basic_command(&mut self) -> sync::Result<ast::BasicCommand, Error> {
		let command = self.parse_argument()
			.with_sync(sync::Strategy::basic_command_terminator())?;

		let pos = command.pos;

		let mut arguments = Vec::new();
		loop {
			let is_redirection = matches!(
				&self.token,
				// The current token is a single unquoted number
				Some(Token { kind: TokenKind::Argument(parts), .. })
					if matches!(parts.as_ref(), &[ref part] if part.is_unquoted_number())
					// And the next token is a redirection operator.
					&& matches!(
						self.cursor.peek(),
						Some(Token { kind: TokenKind::CmdOperator(op), .. })
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

		let (redirections, abort_on_error) = self.parse_operators()
			.with_sync(sync::Strategy::basic_command_terminator())?;


		Ok(
			ast::BasicCommand {
				program: command,
				arguments: arguments.into(),
				redirections: redirections.into(),
				abort_on_error,
				pos,
			}
		)
	}


	/// Parse a single argument.
	fn parse_argument(&mut self) -> Result<ast::Argument, Error> {
		let (token_parts, pos) = self.eat(|token| match token {
			Token { kind: TokenKind::Argument(parts), pos } => Ok((parts, pos)),
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

		let push_dollar = |literal: &mut Vec<u8>, parts: &mut Vec<ast::ArgPart>, symbol, pos| {
			push_literal(literal, parts);
			parts.push(
				ast::ArgPart::Unit(ast::ArgUnit::Dollar { symbol, pos })
			);
		};

		for part in token_parts.into_vec() { // Use vec's owned iterator.
			match part {
				ArgPart::SingleQuoted(lit) => join_literal(&mut literal, lit),

				ArgPart::DoubleQuoted(units) => for unit in units.into_vec() {
					match unit {
						ArgUnit::Dollar { symbol, pos } => push_dollar(&mut literal, &mut parts, symbol, pos),
						// Literals in double quotes don't expand to patterns.
						ArgUnit::Literal(lit) => join_literal(&mut literal, lit),
					}
				}

				ArgPart::Unquoted(unit) => {
					match unit {
						ArgUnit::Dollar { symbol, pos } => push_dollar(&mut literal, &mut parts, symbol, pos),
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


	/// Parse command operators.
	/// Returns a pair of redirections and try operator.
	fn parse_operators(&mut self) -> Result<(Box<[ast::Redirection]>, bool), Error> {
		let mut redirections = Vec::new();

		loop {
			match &self.token {
				Some(Token { kind: token, .. }) if token.is_basic_command_terminator() => break,

				Some(Token { kind: TokenKind::CmdOperator(Operator::Try), .. }) => {
					self.step();

					return Ok((redirections.into(), true));
				}

				Some(_) => {
					let redirection = self.parse_redirection()
						.synchronize(self);

					redirections.push(redirection);
				}

				None => return Err(Error::unexpected_eof()),
			}
		}

		Ok((redirections.into(), false))
	}


	/// Parse a single redirection operation.
	fn parse_redirection(&mut self) -> sync::Result<ast::Redirection, Error> {
		match &self.token {
			// Input redirection.
			&Some(Token { kind: TokenKind::CmdOperator(Operator::Input { literal }), .. }) => {
				self.step();

				let source = self.parse_argument()
					.with_sync(sync::Strategy::keep())?;

				Ok(
					ast::Redirection::Input { literal, source }
				)
			}

			// Expect a output redirection, with an optional prefixed file descriptor.
			Some(_) => {
				let source_fd = self
					.parse_file_descriptor()
					.unwrap_or(io::stdout_fd());

				let redirection = self.parse_output_redirection(source_fd)?;

				Ok(redirection)
			}

			None => Err(Error::unexpected_eof())
				.with_sync(sync::Strategy::eof()),
		}
	}


	/// Parse a single output redirection operation after the optional file descriptor.
	fn parse_output_redirection(
		&mut self, source: FileDescriptor
	) -> sync::Result<ast::Redirection, Error> {
		match &self.token {
			&Some(Token { kind: TokenKind::CmdOperator(Operator::Output { append }), .. }) => {
				self.step();

				let target = if append { // >> file
					let target = self.parse_argument()
						.with_sync(sync::Strategy::keep())?;

					ast::RedirectionTarget::Append(target)
				} else if let Some(fd) = self.parse_file_descriptor() { // > fd
					ast::RedirectionTarget::Fd(fd)
				} else { // > file
					let target = self.parse_argument()
						.with_sync(sync::Strategy::keep())?;

					ast::RedirectionTarget::Overwrite(target)
				};

				Ok(
					ast::Redirection::Output { source, target }
				)
			}

			Some(token) => Err(Error::unexpected_msg(token.clone(), "output redirection"))
				.with_sync(sync::Strategy::skip_one()),

			None => Err(Error::unexpected_eof())
				.with_sync(sync::Strategy::eof()),
		}
	}


	/// Parse a optional file descriptor from a argument.
	fn parse_file_descriptor(&mut self) -> Option<FileDescriptor> {
		match &self.token {
			Some(Token { kind: TokenKind::Argument(parts), .. }) => {
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
}
