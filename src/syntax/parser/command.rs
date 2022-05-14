use crate::{io::{self, FileDescriptor}, semantic::program::SourcePos};
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
		if let Some(Token { kind: TokenKind::CloseCommand, pos }) = &self.token {
			return Err(Error::empty_command_block(*pos))
				.with_sync(sync::Strategy::skip_one())?;
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
		let env = std::iter::from_fn(|| self.parse_env_assign()).collect();

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
				env,
				arguments: arguments.into(),
				redirections,
				abort_on_error,
				pos,
			}
		)
	}


	/// Parse a single argument.
	fn parse_argument(&mut self) -> Result<ast::Argument, Error> {
		let (arg_parts, pos) = self.eat(|token| match token {
			Token { kind: TokenKind::Argument(parts), pos } => Ok((parts, pos)),
			token => Err((Error::unexpected_msg(token.clone(), "argument"), token)),
		})?;

		Ok(
			Self::build_arg(
				arg_parts.into_vec(), // Use vec's owned iterator.
				pos
			)
		)
	}

	/// Parse an env-assign.
	fn parse_env_assign(&mut self) -> Option<(ast::ArgUnit, ast::Argument)> {
		self.eat(|token| match token {
			Token { kind: TokenKind::Argument(parts), pos }
			if matches!(&parts[..], [ ArgPart::Unquoted(_), ArgPart::EnvAssign, .. ]) => {
				let mut parts = parts.into_vec(); // Use vec's owned iterator.

				let value = Self::build_arg(
					parts.drain(2..),
					pos
				);

				let key = match parts.drain(..).next() {
					Some(ArgPart::Unquoted(key)) => Self::build_arg_unit(key),
					_ => unreachable!("pattern matched key is missing"),
				};

				Ok(Some((key, value)))
			},
			token => Err((Error::InvalidEnvAssign, token)),
		})
			.unwrap_or(None)
	}

	/// Parse command operators.
	/// Returns a pair of (redirections, abort_on_error).
	fn parse_operators(&mut self) -> Result<(Box<[ast::Redirection]>, bool), Error> {
		let mut redirections = Vec::new();

		loop {
			match &self.token {
				Some(Token { kind: token, .. }) if token.is_basic_command_terminator() => break,

				Some(Token { kind: TokenKind::CmdOperator(Operator::Try), .. }) => {
					self.step();

					return Ok((redirections.into(), false));
				}

				Some(_) => {
					let redirection = self.parse_redirection()
						.synchronize(self);

					redirections.push(redirection);
				}

				None => return Err(Error::unexpected_eof()),
			}
		}

		Ok((redirections.into(), true))
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
					.unwrap_or_else(io::stdout_fd);

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
					[ArgPart::Unquoted(ArgUnit::Literal(ref lit))] => {
						let lit = std::str::from_utf8(lit).ok()?;
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

	fn build_arg<J>(arg_parts: J, pos: SourcePos) -> ast::Argument
	where
		J: IntoIterator<Item = ArgPart>,
	{
		let mut parts = Vec::<ast::ArgPart>::new();
		let mut literal = Vec::<u8>::new();

		let join_owned_literal = |literal: &mut Vec<u8>, lit: Box<[u8]>| {
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

		let push_part = |literal: &mut Vec<u8>, parts: &mut Vec<ast::ArgPart>, part| {
			push_literal(literal, parts);
			parts.push(part);
		};

		let push_dollar = |literal: &mut Vec<u8>, parts: &mut Vec<ast::ArgPart>, symbol, pos| {
			push_part(
				literal,
				parts,
				ast::ArgPart::Unit(ast::ArgUnit::Dollar { symbol, pos })
			);
		};

		for part in arg_parts {
			match part {
				ArgPart::SingleQuoted(lit) => join_owned_literal(&mut literal, lit),

				ArgPart::DoubleQuoted(units) => for unit in units.into_vec() {
					match unit {
						ArgUnit::Dollar { symbol, pos } => push_dollar(&mut literal, &mut parts, symbol, pos),
						// Literals in double quotes don't expand to patterns.
						ArgUnit::Literal(lit) => join_owned_literal(&mut literal, lit),
					}
				}

				ArgPart::Unquoted(unit) => {
					match unit {
						ArgUnit::Dollar { symbol, pos } => push_dollar(&mut literal, &mut parts, symbol, pos),
						ArgUnit::Literal(lit) => join_owned_literal(&mut literal, lit),
					}
				}

				ArgPart::Expansion(expansion) => push_part(
					&mut literal,
					&mut parts,
					ast::ArgPart::Expansion(expansion.into())
				),

				// Env assign past the first command should be treated as a literal.
				ArgPart::EnvAssign => literal.extend(b"="),
			}
		}

		// Push the trailing literal, if any.
		push_literal(&mut literal, &mut parts);

		ast::Argument {
			parts: parts.into(),
			pos
		}
	}

	fn build_arg_unit(unit: ArgUnit) -> ast::ArgUnit {
		match unit {
			ArgUnit::Dollar { symbol, pos } => ast::ArgUnit::Dollar { symbol, pos },
			ArgUnit::Literal(lit) => ast::ArgUnit::Literal(lit),
		}
	}
}
