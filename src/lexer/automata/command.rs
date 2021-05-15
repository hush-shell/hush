use super::{
	Arg,
	symbol::CommandSymbolChar,
	Root,
	TokenKind,
	Comment,
	Cursor,
	State,
	CommandSymbol,
	Token,
	Transition,
};


#[derive(Debug)]
pub(super) struct Command;


impl Command {
	pub fn visit<'a>(self, cursor: &Cursor<'a>) -> Transition<'a> {
		match cursor.peek() {
			Some(c) if c.is_ascii_whitespace() => Transition::step(self),

			Some(b'#') => Transition::step(Comment::from(self)),

			Some(b'}') => Transition::produce(
				Root,
				Token {
					token: TokenKind::CloseCommand,
					pos: cursor.pos(),
				}
			),

			Some(c) => match CommandSymbolChar::from_first(c) {
				CommandSymbolChar::None => Transition::resume(Arg::at(cursor)), // Argument.

				CommandSymbolChar::Single(token) => { // Semicolon, pipe or try.
					Transition::produce(self, Token { token, pos: cursor.pos() })
				}

				// >, >>, <, <<
				CommandSymbolChar::Double { first } => Transition::step(CommandSymbol::from_first(first, cursor)),
			},

			None => Transition::step(self),
		}
	}
}


impl<'a> From<Command> for State<'a> {
	fn from(state: Command) -> State<'a> {
		State::Command(state)
	}
}
