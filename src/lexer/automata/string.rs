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
		let literal = |literal| Token { token: TokenKind::Literal(literal), pos: self.pos };

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
			(&Self { error: Some(err), .. }, Some(b'\'')) => {
				error(
					ErrorKind::InvalidLiteral(invalid_literal(err))
				)
			}

			// Value has been scanned, and there have been no errors.
			(&Self { value: Some(value), .. }, Some(b'\'')) => {
				produce(literal(Literal::Byte(value)))
			}

			// If a value has already been scanned (including incorrect escape sequences). There
			// should be no further characters except for the closing quote.
			(&Self { value: Some(_), .. }, Some(c)) |
			(&Self { error: Some(_), .. }, Some(c)) => {
				error(ErrorKind::Unexpected(c))
			}

			// Escaped character.
			(&Self { escaping: Some(escape_offset), .. }, Some(value)) => {
				match value {
					b'"'  => self.value = Some(b'"'),
					b'\'' => self.value = Some(b'\''),
					b'n'  => self.value = Some(b'\n'),
					b't'  => self.value = Some(b'\t'),
					b'0'  => self.value = Some(b'\0'),
					_ => self.error = Some( // Invalid escape sequence.
						&cursor.slice()[escape_offset ..= cursor.offset()]
					)
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
	/// The parsed bytes, if any.
	value: Vec<u8>,
	/// Escape sequence errors, if any.
	errors: Vec<InvalidEscapeCode<'a>>,
	/// The position of the current escape sequence, if any.
	escaping: Option<usize>,
	/// The position of the literal.
	pos: SourcePos,
}


impl<'a> StringLiteral<'a> {
	pub fn at(cursor: &Cursor) -> Self {
		StringLiteral {
			value: Vec::with_capacity(10), // We expect most literals to not be empty.
			errors: Vec::new(), // Don't allocate until we meet errors.
			escaping: None,
			pos: cursor.pos(),
		}
	}


	pub fn visit(mut self, cursor: &Cursor<'a>) -> Transition<'a> {
		let produce = |token| Transition::produce(Root, token);

		macro_rules! literal {
			($literal:expr) => {
				Token { token: TokenKind::Literal($literal), pos: self.pos }
			}
		}

		macro_rules! error {
			($error:expr) => {
				Transition::error(
					Root,
					Error {
						error: $error,
						pos: self.pos,
					}
				)
			}
		}

		match (&self, cursor.peek()) {
			// EOF while scanning a literal is always an error.
			(_, None) => {
				error!(ErrorKind::UnexpectedEof)
			}

			// Escaped character.
			(&Self { escaping: Some(escape_offset), .. }, Some(value)) => {
				match value {
					b'"'  => self.value.push(b'"'),
					b'\'' => self.value.push(b'\''),
					b'n'  => self.value.push(b'\n'),
					b't'  => self.value.push(b'\t'),
					b'0'  => self.value.push(b'\0'),
					_ => self.errors.push( // Invalid escape sequence.
						&cursor.slice()[escape_offset ..= cursor.offset()]
					),
				};
				self.escaping = None;
				Transition::step(self)
			}

			// Begin of escape sequence.
			(_, Some(b'\\')) => {
				self.escaping = Some(cursor.offset());
				Transition::step(self)
			}

			// End of literal.
			(_, Some(b'\"')) => {
				if self.errors.is_empty() {
					produce(literal!(Literal::String(self.value.into())))
				} else {
					error!(
						ErrorKind::InvalidLiteral(
							InvalidLiteral::InvalidEscapeCodes(
								self.errors.into_boxed_slice()
							)
						)
					)
				}
			}

			// Ordinary character.
			(_, Some(value)) => {
				self.value.push(value);
				Transition::step(self)
			}
		}
	}
}


impl<'a> From<StringLiteral<'a>> for State<'a> {
	fn from(state: StringLiteral<'a>) -> State<'a> {
		State::StringLiteral(state)
	}
}
