use super::{
	symbol::CommandSymbolChar,
	Argument,
	CommandSymbol,
	Comment,
	Cursor,
	Error,
	Root,
	State,
	Token,
	TokenKind,
	Transition,
};


/// The state for lexing command blocks.
#[derive(Debug)]
pub(super) struct Command;


impl Command {
	pub fn visit(self, cursor: &Cursor) -> Transition {
		match cursor.peek() {
			// Whitespace.
			Some(c) if c.is_ascii_whitespace() => Transition::step(self),

			// Comment.
			Some(b'#') => Transition::step(Comment::from(self)),

			// Close command block.
			Some(b'}') => Transition::produce(
				Root,
				Token { token: TokenKind::CloseCommand, pos: cursor.pos() },
			),

			// Argument or operator.
			Some(c) => match CommandSymbolChar::from_first(c) {
				// Argument.
				CommandSymbolChar::None => Transition::resume(Argument::at(cursor)),

				// Semicolon, pipe or try.
				CommandSymbolChar::Single(token) => {
					Transition::produce(self, Token { token, pos: cursor.pos() })
				}

				// >, >>, <, <<.
				CommandSymbolChar::Double { first } => {
					Transition::step(CommandSymbol::from_first(first, cursor))
				}
			},

			// Eof.
			None => Transition::error(Root, Error::unexpected_eof(cursor.pos())),
		}
	}
}


impl From<Command> for State {
	fn from(state: Command) -> State {
		State::Command(state)
	}
}
