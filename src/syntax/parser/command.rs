use super::{
	ast,
	CommandOperator,
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
		while let Ok(arg) = self.parse_argument() {
			arguments.push(arg);
		}

		let redirections = self.parse_redirections()?;

		let abort_on_error = matches!(
			&self.token,
			Some(Token { token: TokenKind::CommandOperator(CommandOperator::Try), .. })
		);
		if abort_on_error {
			self.step();
		}

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
		let (parts, pos) = self.eat(|token| match token {
			Token { token: TokenKind::Argument(parts), pos } => Ok((parts, pos)),
			token => Err((Error::unexpected_msg(token.clone(), "argument"), token)),
		})?;

		Ok(ast::Argument::from_arg_parts(parts, pos))
	}


	fn parse_redirections(&mut self) -> Result<Box<[ast::Redirection]>, Error> {
		// TODO
		// > file
		// 1> file
		// 2> 1
		// 1> "2"
		// < file
		// << input
		Ok(
			Box::default()
		)
	}

	/// Semicolon-separated items.
	fn semicolon_sep<P, R>(&mut self, parse: P) -> Result<Box<[R]>, Error>
	where
		P: FnMut(&mut Self) -> Result<R, Error>,
	{
		self.sep_by(parse, |token| *token == TokenKind::Semicolon)
	}
}
