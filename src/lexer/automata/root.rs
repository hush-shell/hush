use super::{
	symbol::SymbolChar,
	word::IsWord,
	Command,
	TokenKind,
	ByteLiteral,
	Comment,
	Cursor,
	Error,
	NumberLiteral,
	State,
	StringLiteral,
	Symbol,
	Token,
	Transition,
	Word,
};


#[derive(Debug)]
pub(super) struct Root;


impl Root {
	pub fn visit<'a>(self, cursor: &Cursor<'a>) -> Transition<'a> {
		match cursor.peek() {
			Some(c) if c.is_ascii_whitespace() => Transition::step(self),

			Some(b'#') => Transition::step(Comment::from(self)),

			Some(b'"') => Transition::step(StringLiteral::at(cursor)),

			Some(b'\'') => Transition::step(ByteLiteral::at(cursor)),

			Some(c) if c.is_ascii_digit() => Transition::step(NumberLiteral::at(cursor)),

			Some(c) if c.is_word() => Transition::resume(Word::at(cursor)),

			Some(c) => match SymbolChar::from_first(c) {
				SymbolChar::None => Transition::error(self, Error::unexpected(c, cursor.pos())),

				SymbolChar::Single(TokenKind::Command) => {
					Transition::produce(
						Command,
						Token { token:TokenKind::Command, pos: cursor.pos() }
					)
				}

				SymbolChar::Single(token) => {
					Transition::produce(self, Token { token, pos: cursor.pos() })
				}

				SymbolChar::Double { first } => Transition::step(Symbol::from_first(first, cursor)),
			},

			None => Transition::step(self),
		}
	}
}


impl<'a> From<Root> for State<'a> {
	fn from(state: Root) -> State<'a> {
		State::Root(state)
	}
}
