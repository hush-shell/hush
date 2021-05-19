use super::{Cursor, Error, Literal, Root, SourcePos, State, Token, TokenKind, Transition};


/// The state for lexing byte literals.
#[derive(Debug)]
pub(super) struct ByteLiteral {
	/// The parsed value, if any.
	value: Option<u8>,
	/// The position of the current escape sequence, if any.
	escaping: Option<(usize, SourcePos)>,
	/// The position of the literal.
	pos: SourcePos,
}


impl ByteLiteral {
	pub fn at(cursor: &Cursor) -> Self {
		Self { value: None, escaping: None, pos: cursor.pos() }
	}


	pub fn visit<'a>(mut self, cursor: &Cursor<'a>) -> Transition<'a> {
		match (&self, cursor.peek()) {
			// EOF while scanning a literal is always an error.
			(_, None) => Transition::error(Root, Error::unexpected_eof(cursor.pos())),

			// Closing quote.
			(&Self { value: Some(c), .. }, Some(b'\'')) => Transition::produce(
				Root,
				Token {
					token: TokenKind::Literal(Literal::Byte(c)),
					pos: self.pos,
				},
			),

			// Empty literal.
			(&Self { value: None, .. }, Some(b'\'')) => {
				Transition::error(Root, Error::empty_byte_literal(self.pos))
			}

			// If a value has already been scanned (including incorrect escape sequences). There
			// should be no further characters except for the closing quote.
			(&Self { value: Some(_), .. }, Some(c)) => {
				Transition::error(self, Error::unexpected(c, cursor.pos()))
			}

			// Escaped character.
			(&Self { escaping: Some((offset, pos)), .. }, Some(value)) => {
				self.escaping = None;

				if let Some(c) = validate_escape(value) {
					self.value = Some(c);
					Transition::step(self)
				} else {
					// Use a placeholder to produce a valid literal after reporting the error. This
					// won't get to be actually used, because the program won't be interpreted after
					// parsing.
					self.value = Some(b'\0');
					let escape_sequence = &cursor.slice()[offset ..= cursor.offset()];
					Transition::error(self, Error::invalid_escape_sequence(escape_sequence, pos))
				}
			}

			// Begin of escape sequence.
			(_, Some(b'\\')) => {
				self.escaping = Some((cursor.offset(), cursor.pos()));
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


impl From<ByteLiteral> for State {
	fn from(state: ByteLiteral) -> State {
		State::ByteLiteral(state)
	}
}


/// The state for lexing string literals.
#[derive(Debug)]
pub(super) struct StringLiteral {
	/// The parsed bytes, if any.
	value: Vec<u8>,
	/// The position of the current escape sequence, if any.
	escaping: Option<(usize, SourcePos)>,
	/// The position of the literal.
	pos: SourcePos,
}


impl StringLiteral {
	pub fn at(cursor: &Cursor) -> Self {
		Self {
			value: Vec::with_capacity(8), // We expect most literals to not be empty.
			escaping: None,
			pos: cursor.pos(),
		}
	}


	pub fn visit<'a>(mut self, cursor: &Cursor<'a>) -> Transition<'a> {
		match (&self, cursor.peek()) {
			// EOF while scanning a literal is always an error.
			(_, None) => Transition::error(Root, Error::unexpected_eof(cursor.pos())),

			// Escaped character.
			(&Self { escaping: Some((offset, pos)), .. }, Some(value)) => {
				self.escaping = None;

				if let Some(c) = validate_escape(value) {
					self.value.push(c);
					Transition::step(self)
				} else {
					let escape_sequence = &cursor.slice()[offset ..= cursor.offset()];
					Transition::error(self, Error::invalid_escape_sequence(escape_sequence, pos))
				}
			}

			// Begin of escape sequence.
			(_, Some(b'\\')) => {
				self.escaping = Some((cursor.offset(), cursor.pos()));
				Transition::step(self)
			}

			// Closing quote.
			(_, Some(b'\"')) => Transition::produce(
				Root,
				Token {
					token: TokenKind::Literal(Literal::String(self.value.into_boxed_slice())),
					pos: self.pos,
				},
			),

			// Ordinary character.
			(_, Some(value)) => {
				self.value.push(value);
				Transition::step(self)
			}
		}
	}
}


impl From<StringLiteral> for State {
	fn from(state: StringLiteral) -> State {
		State::StringLiteral(state)
	}
}


/// Check if a escape sequence is valid, producing the correspondent byte if so.
fn validate_escape(sequence: u8) -> Option<u8> {
	match sequence {
		b'"' => Some(b'"'),
		b'\'' => Some(b'\''),
		b'n' => Some(b'\n'),
		b't' => Some(b'\t'),
		b'0' => Some(b'\0'),
		b'\\' => Some(b'\\'),
		_ => None,
	}
}
