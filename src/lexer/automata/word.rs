use super::{
	Cursor,
	Keyword,
	Literal,
	Operator,
	Root,
	SourcePos,
	State,
	SymbolInterner,
	Token,
	TokenKind,
	Transition,
};


#[derive(Debug)]
pub struct Word {
	start_offset: usize,
	pos: SourcePos,
}


pub trait IsWord {
	fn is_word(&self) -> bool;
}


impl IsWord for u8 {
	fn is_word(&self) -> bool {
		self.is_ascii_alphanumeric() || *self == b'_'
	}
}


impl Word {
	pub fn at(cursor: &Cursor) -> Self {
		Self {
			start_offset: cursor.offset(),
			pos: cursor.pos()
		}
	}


	pub fn visit<'a>(
		self,
		cursor: &Cursor<'a>,
		interner: &mut SymbolInterner
	) -> Transition<'a> {
		match cursor.peek() {
			Some(c) if c.is_word() => Transition::step(self),
			// If we visit EOF or a non-identifier character, we should just produce.
			_ => {
				let word = cursor.slice(self.start_offset, cursor.offset());
				let token = Self::to_token(word, interner);

				Transition::revisit_produce(
					Root,
					Token { token, pos: self.pos }
				)
			}
		}
	}


	fn to_token(word: &[u8], interner: &mut SymbolInterner) -> TokenKind {
		match word {
			// Keywords:
			b"let" => TokenKind::Keyword(Keyword::Let),
			b"if" => TokenKind::Keyword(Keyword::If),
			b"then" => TokenKind::Keyword(Keyword::Then),
			b"else" => TokenKind::Keyword(Keyword::Else),
			b"end" => TokenKind::Keyword(Keyword::End),
			b"for" => TokenKind::Keyword(Keyword::For),
			b"in" => TokenKind::Keyword(Keyword::In),
			b"do" => TokenKind::Keyword(Keyword::Do),
			b"while" => TokenKind::Keyword(Keyword::While),
			b"function" => TokenKind::Keyword(Keyword::Function),
			b"return" => TokenKind::Keyword(Keyword::Return),
			b"break" => TokenKind::Keyword(Keyword::Break),
			b"self" => TokenKind::Keyword(Keyword::Self_),

			// Literals:
			b"nil" => TokenKind::Literal(Literal::Nil),
			b"true" => TokenKind::Literal(Literal::True),
			b"false" => TokenKind::Literal(Literal::False),

			// Operators:
			b"not" => TokenKind::Operator(Operator::Not),
			b"and" => TokenKind::Operator(Operator::And),
			b"or" => TokenKind::Operator(Operator::Or),

			// Identifier:
			ident => {
				let ident = unsafe {
					// As identifiers are composed only of alphanumeric or underscore characters,
					// and we have checked those, they are valid utf8.
					std::str::from_utf8_unchecked(ident)
				};
				let symbol = interner.get_or_intern(ident);
				TokenKind::Identifier(symbol)
			}
		}
	}
}


impl<'a> From<Word> for State<'a> {
	fn from(state: Word) -> State<'a> {
		State::Word(state)
	}
}
