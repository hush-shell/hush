use super::{
	symbol::SymbolChar,
	word::IsWord,
	ByteLiteral,
	Command,
	Comment,
	Cursor,
	Error,
	NumberLiteral,
	State,
	StringLiteral,
	Symbol,
	Token,
	TokenKind,
	Transition,
	Word,
};


/// The top level lexer state.
#[derive(Debug)]
pub(super) struct Root;


impl Root {
	pub fn visit(self, cursor: &Cursor) -> Transition {
		match cursor.peek() {
			// Whitespace.
			Some(c) if c.is_ascii_whitespace() => Transition::step(self),

			// Comments.
			Some(b'#') => Transition::step(Comment::from(self)),

			// String literals.
			Some(b'"') => Transition::step(StringLiteral::at(cursor)),

			// Byte literals.
			Some(b'\'') => Transition::step(ByteLiteral::at(cursor)),

			// Number literals.
			Some(c) if c.is_ascii_digit() => Transition::step(NumberLiteral::at(cursor)),

			// Identifier, keywords and word operators.
			Some(c) if c.is_word_start() => Transition::resume(Word::at(cursor)),

			// Symbols.
			Some(c) => match SymbolChar::from_first(c) {
				SymbolChar::None => Transition::error(self, Error::unexpected(c, cursor.pos())),

				SymbolChar::Single(TokenKind::Command) => Transition::produce(
					Command,
					Token { token: TokenKind::Command, pos: cursor.pos() },
				),

				SymbolChar::Single(token) => {
					Transition::produce(self, Token { token, pos: cursor.pos() })
				}

				SymbolChar::Double { first } => Transition::step(Symbol::from_first(first, cursor)),
			},

			// Eof.
			None => Transition::step(self),
		}
	}
}


impl From<Root> for State {
	fn from(state: Root) -> State {
		State::Root(state)
	}
}
