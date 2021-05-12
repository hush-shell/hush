use super::{
	Error,
	ErrorKind,
	Cursor,
	Operator,
	Root,
	SourcePos,
	State,
	Token,
	TokenKind,
	Transition,
};


#[derive(Debug)]
pub struct Symbol {
	first: u8,
	pos: SourcePos,
}


impl Symbol {
	pub fn from_first(first: u8, cursor: &Cursor) -> Self {
		Self {
			first,
			pos: cursor.pos(),
		}
	}


	pub fn visit<'a>(self, cursor: &Cursor<'a>) -> Transition<'a> {
		let unexpected = |input| Transition::skip_error(Root, Error::unexpected(input, self.pos));
		let token = |token| Token { token, pos: self.pos };
		let operator = |op| token(TokenKind::Operator(op));

		let produce = |token| Transition::produce(Root, token);
		let skip_produce = |output| Transition::revisit_produce(Root, output);

		match (self.first, cursor.peek()) {
			(b'>', Some(b'=')) => produce(operator(Operator::GreaterEquals)),
			(b'>', _) => skip_produce(operator(Operator::Greater)),

			(b'<', Some(b'=')) => produce(operator(Operator::LowerEquals)),
			(b'<', _) => skip_produce(operator(Operator::Lower)),

			(b'+', Some(b'+')) => produce(operator(Operator::Concat)),
			(b'+', _) => skip_produce(operator(Operator::Plus)),

			(b'=', Some(b'=')) => produce(operator(Operator::Equals)),
			(b'=', _) => skip_produce(operator(Operator::Assign)),

			(b'!', Some(b'=')) => produce(operator(Operator::NotEquals)),
			(b'!', _) => unexpected(self.first),

			(b'@', Some(b'[')) => produce(token(TokenKind::OpenDict)),
			(b'@', _) => unexpected(self.first),

			(b'$', Some(b'{')) => produce(token(TokenKind::CaptureCommand)),
			(b'$', _) => unexpected(self.first),

			(b'&', Some(b'{')) => produce(token(TokenKind::AsyncCommand)),
			(b'&', _) => unexpected(self.first),

			// We must have covered all possibilites for the first character. The peeked
			// character is wildcarded, which will cover everthing including EOF (None).
			_ => unreachable!("invalid first character in symbol state"),
		}
	}
}


impl<'a> From<Symbol> for State<'a> {
	fn from(state: Symbol) -> State<'a> {
		State::Symbol(state)
	}
}


/// Helper for special characters.
pub enum SymbolChar {
	/// Not a symbol character.
	None,
	/// Some symbols are single characters. We can produce them straight away.
	Single(TokenKind),
	/// Others have two characters, so we must handle those separately.
	Double { first: u8 },
}


impl SymbolChar {
	pub fn from_first(first: u8) -> Self {
		let token = |token| Self::Single(token);
		let operator = |op| token(TokenKind::Operator(op));
		let double = |c| Self::Double { first: c };

		match first {
			// Single character.
			b'-' => operator(Operator::Minus),
			b'*' => operator(Operator::Times),
			b'/' => operator(Operator::Div),
			b'%' => operator(Operator::Mod),
			b'.' => operator(Operator::Dot),
			b':' => token(TokenKind::Colon),
			b',' => token(TokenKind::Comma),
			b'(' => token(TokenKind::OpenParens),
			b')' => token(TokenKind::CloseParens),
			b'[' => token(TokenKind::OpenBracket),
			b']' => token(TokenKind::CloseBracket),
			b'{' => token(TokenKind::Command),

			// Double character.
			b'>' => double(first),
			b'<' => double(first),
			b'+' => double(first),
			b'=' => double(first),
			b'!' => double(first),
			b'@' => double(first),
			b'$' => double(first),
			b'&' => double(first),

			// Not a symbol character:
			_ => SymbolChar::None
		}
	}
}
