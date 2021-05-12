use super::{
	Cursor,
	Error,
	ErrorKind,
	InvalidEscapeCode,
	InvalidLiteral,
	Literal,
	Root,
	SourcePos,
	State,
	Token,
	TokenKind,
	Transition,
};


#[derive(Debug)]
pub struct ByteLiteral<'a> {
	/// The parsed value, if any.
	value: Option<u8>,
	/// Escape sequence error, if any.
	error: Option<InvalidEscapeCode<'a>>,
	/// The position of the current escape sequence, if any.
	escaping: Option<usize>,
	/// The position of the literal.
	pos: SourcePos,
}


impl<'a> ByteLiteral<'a> {
	pub fn at(cursor: &Cursor) -> Self {
		ByteLiteral {
			value: None,
			error: None,
			escaping: None,
			pos: cursor.pos(),
		}
	}


	pub fn visit(mut self, cursor: &Cursor<'a>) -> Transition<'a> {
		type State<'a> = ByteLiteral<'a>;

		let token = |token| Token { token, pos: self.pos };
		let literal = |literal| token(TokenKind::Literal(literal));

		let produce = |token| Transition::produce(Root, token);
		let error = |error| Transition::error(
			Root,
			Error {
				error,
				pos: self.pos,
			}
		);
		let invalid_literal = |escape_code| InvalidLiteral::InvalidEscapeCodes(
			Box::new([escape_code])
		);

		match (&self, cursor.peek()) {
			// EOF while scanning a literal is always an error.
			(_, None) => {
				error(ErrorKind::UnexpectedEof)
			}

			// If there has been some previous error, consume the closing quote before
			// reporting.
			(&State { error: Some(err), .. }, Some(b'\'')) => {
				error(
					ErrorKind::InvalidLiteral(invalid_literal(err))
				)
			}

			// Value has been scanned, and there have been no errors.
			(&State { value: Some(value), .. }, Some(b'\'')) => {
				produce(literal(Literal::Byte(value)))
			}

			// If a value has already been scanned (including incorrect escape sequences). There
			// should be no further characters except for the closing quote.
			(&State { value: Some(_), .. }, Some(c)) |
			(&State { error: Some(_), .. }, Some(c)) => {
				error(ErrorKind::Unexpected(c))
			}

			// Escaped character.
			(&State { escaping: Some(escape_offset), .. }, Some(value)) => {
				self.value = match value {
					b'"'  => Some(b'"'),
					b'\'' => Some(b'\''),
					b'n'  => Some(b'\n'),
					b't'  => Some(b'\t'),
					b'0'  => Some(b'\0'),
					_ => {
						self.error = Some(
							&cursor.slice()[escape_offset ..= cursor.offset()]
						);
						None
					}
				};
				self.escaping = None;
				Transition::step(self)
			}

			// Begin of escape sequence.
			(_, Some(b'\\')) => {
				self.escaping = Some(cursor.offset());
				Transition::step(self)
			}

			// Ordinary character.
			(_, Some(value)) => {
				self.value = Some(value);
				Transition::step(self)
			}
		}
	}
}


impl<'a> From<ByteLiteral<'a>> for State<'a> {
	fn from(state: ByteLiteral<'a>) -> State<'a> {
		State::ByteLiteral(state)
	}
}


#[derive(Debug)]
pub struct StringLiteral<'a> {
	value: Vec<u8>,
	errors: Vec<InvalidEscapeCode<'a>>,
	escaping: bool,
	pos: SourcePos,
}


impl<'a> StringLiteral<'a> {
	pub fn at(cursor: &Cursor) -> Self {
		StringLiteral {
			value: Vec::with_capacity(10), // We expect most literals to not be empty.
			errors: Vec::new(), // Don't allocate until we meet errors.
			escaping: false,
			pos: cursor.pos(),
		}
	}


	pub fn visit(self, cursor: &Cursor<'a>) -> Transition<'a> {
		todo!()
	}
}


impl<'a> From<StringLiteral<'a>> for State<'a> {
	fn from(state: StringLiteral<'a>) -> State<'a> {
		State::StringLiteral(state)
	}
}
