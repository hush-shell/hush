use super::{
	symbol::SymbolChar,
	word::IsWord,
	Comment,
	Cursor,
	Error,
	State,
	Token,
	Transition,
	StringLiteral,
	ByteLiteral,
	Symbol,
	Word,
};


#[derive(Debug)]
pub struct Root;


impl Root {
	pub fn visit<'a>(self, cursor: &Cursor<'a>) -> Transition<'a> {
		match cursor.peek() {
			Some(c) if c.is_ascii_whitespace() => {
				Transition::step(self)
			}

			Some(b'#') => {
				Transition::step(Comment)
			}

			Some(b'"') => {
				Transition::step(StringLiteral::at(cursor))
			}

			Some(b'\'') => {
				Transition::step(ByteLiteral::at(cursor))
			}

			Some(c) if c.is_word() => {
				Transition::revisit(Word::at(cursor))
			}

			Some(c) => {
				match SymbolChar::from_first(c) {
					SymbolChar::None => Transition::error(
						self,
						Error::unexpected(c, cursor.pos())
					),

					SymbolChar::Single(token) => Transition::produce(
						self,
						Token { token, pos: cursor.pos() }
					),

					SymbolChar::Double { first } => Transition::step(
						Symbol::from_first(first, cursor)
					),
				}
			}

			None => {
				Transition::step(self)
			}
		}
	}
}


impl<'a> From<Root> for State<'a> {
	fn from(state: Root) -> State<'a> {
		State::Root(state)
	}
}
