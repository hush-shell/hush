use super::{
	InvalidEscapeCode,
	InvalidLiteral,
	Error,
	ErrorKind,
	Cursor,
	Command,
	SourcePos,
	Transition,
	State,
	Token,
	TokenKind,
	Argument,
	BasicArgument
};


pub(super) trait WordContext<'a> {
	/// The transition to make when the argument has been consumed.
	fn resume(self, value: Vec<u8>) -> Transition<'a>;
	/// The transition to signal errors.
	fn resume_error(self, errors: Vec<InvalidEscapeCode<'a>>) -> Transition<'a>;
	/// Check if a character should be consumed.
	fn is_word(value: u8) -> bool;
	/// Check if a character is a valid escape sequence, and return it's corresponding
	/// value.
	fn validate_escape(value: u8) -> Option<u8>;
}


/// This state should be introduced only when the next character is a valid argument
/// starter: a quote or a word character.
#[derive(Debug)]
pub(super) struct Arg<'a> {
	parts: Vec<Argument>,
	errors: Vec<InvalidEscapeCode<'a>>,
	pos: SourcePos,
}


impl<'a> Arg<'a> {
	pub fn at(cursor: &Cursor) -> Self {
		Self {
			parts: Vec::with_capacity(1), // Any arg should have at least one part.
			errors: Vec::new(),
			pos: cursor.pos(),
		}
	}


	pub fn visit(self, cursor: &Cursor<'a>) -> Transition<'a> {
		match cursor.peek() {
			Some(b'\'') => Transition::step(SingleQuoted::from(self)),

			Some(b'"') => Transition::step(DoubleQuoted::from(self)),

			Some(b'$') => todo!(),

			Some(c) if Self::is_word(c) => Transition::resume(Word::from(self)),

			Some(_) => {
				// The Arg state must be introduced only when an argument character is found, and
				// therefore it should have at least before one part before a whitespace is visited.
				debug_assert!(!self.parts.is_empty());

				if self.errors.is_empty() {
					Transition::resume_produce(
						Command,
						Token {
							token: TokenKind::Argument(self.parts.into_boxed_slice()),
							pos: self.pos,
						}
					)
				} else {
					Transition::resume_error(
						Command,
						Error {
							error: ErrorKind::InvalidLiteral(
								InvalidLiteral::InvalidEscapeCodes(self.errors.into_boxed_slice())
							),
							pos: self.pos,
						}
					)
				}
			}

			None => Transition::error(
				self,
				Error {
					error: ErrorKind::UnexpectedEof,
					pos: cursor.pos(),
				}
			)
		}
	}
}


impl<'a> WordContext<'a> for Arg<'a> {
	fn resume(mut self, value: Vec<u8>) -> Transition<'a> {
		self.parts.push(
			Argument::Unquoted(
				BasicArgument::Literal(value.into_boxed_slice())
			)
		);

		Transition::resume(self)
	}

	fn resume_error(mut self, errors: Vec<InvalidEscapeCode<'a>>) -> Transition<'a> {
		self.errors.extend(errors);
		Transition::resume(self)
	}

	fn is_word(value: u8) -> bool {
		match value {
			b'#' => false, // Comments.
			b'\'' | b'"' => false, // Quotes.
			b'>' | b'<' | b'?' | b';' => false, // Symbols.
			b'$' => false, // Dollar.
			c if c.is_ascii_whitespace() => false,
			_ => true,
		}
	}

	fn validate_escape(value: u8) -> Option<u8> {
		match value {
			// Syntactical escape sequences:
			b'#' => Some(value), // Escaped number sign.
			b'\'' | b'"' => Some(value), // Escaped quotes.
			b'>' | b'<' | b'?' | b';' => Some(value), // Escaped symbols.
			b'$' => Some(value), // Escaped dollar.
			c if c.is_ascii_whitespace() => Some(value), // Escaped whitespace.

			// Additional escape sequences:
			b'n' => Some(b'\n'),
			b't' => Some(b'\t'),
			b'\\' => Some(b'\\'),

			// Invalid escape sequence:
			_ => None,
		}
	}
}


impl<'a> From<Arg<'a>> for State<'a> {
	fn from(state: Arg<'a>) -> State<'a> {
		State::Argument(state)
	}
}


#[derive(Debug)]
pub(super) struct SingleQuoted<'a> {
	/// The parsed bytes, if any.
	value: Vec<u8>,
	/// Errors, if any.
	errors: Vec<InvalidEscapeCode<'a>>,
	/// The parent state.
	parent: Arg<'a>,
}


impl<'a> SingleQuoted<'a> {
	pub fn visit(mut self, cursor: &Cursor<'a>) -> Transition<'a> {
		match cursor.peek() {
			// End of literal.
			Some(b'\'') => {
				self.parent.parts.push(
					Argument::SingleQuoted(self.value.into_boxed_slice())
				);

				Transition::step(self.parent)
			}

			// This must be the start of the literal, because the WordContext instance for
			// SingleQuoted guarantees that the only non-word character is the closing quote.
			Some(_) => Transition::resume(Word::from(self)),

			None => Transition::error(
				self,
				Error {
					error: ErrorKind::UnexpectedEof,
					pos: cursor.pos(),
				}
			)
		}
	}
}


impl<'a> From<Arg<'a>> for SingleQuoted<'a> {
	fn from(parent: Arg<'a>) -> Self {
		Self {
			value: Vec::with_capacity(10), // We expect most literals to not be empty.
			errors: Vec::new(),
			parent,
		}
	}
}


impl<'a> WordContext<'a> for SingleQuoted<'a> {
	fn resume(mut self, value: Vec<u8>) -> Transition<'a> {
		self.value = value;
		Transition::resume(self)
	}

	fn resume_error(mut self, errors: Vec<InvalidEscapeCode<'a>>) -> Transition<'a> {
		self.errors.extend(errors);
		Transition::resume(self)
	}

	fn is_word(value: u8) -> bool {
		// Comments, double quotes, symbols, dollars and whitespace are literals in single
		// quotes.
		value != b'\''
	}

	fn validate_escape(value: u8) -> Option<u8> {
		match value {
			// Syntactical escape sequences:
			b'\'' => Some(value), // Escaped quotes.

			// Additional escape sequences:
			b'n' => Some(b'\n'),
			b't' => Some(b'\t'),
			b'\\' => Some(b'\\'),

			// Invalid escape sequence:
			_ => None,
		}
	}
}


impl<'a> From<SingleQuoted<'a>> for State<'a> {
	fn from(state: SingleQuoted<'a>) -> State<'a> {
		State::SingleQuoted(state)
	}
}


#[derive(Debug)]
pub(super) struct DoubleQuoted<'a> {
	// The parts of the literal.
	parts: Vec<BasicArgument>,
	/// Errors, if any.
	errors: Vec<InvalidEscapeCode<'a>>,
	/// The parent state.
	parent: Arg<'a>,
}


impl<'a> DoubleQuoted<'a> {
	pub fn visit(mut self, cursor: &Cursor<'a>) -> Transition<'a> {
		match cursor.peek() {
			Some(b'\"') => {
				self.parent.parts.push(
					Argument::DoubleQuoted(self.parts.into_boxed_slice())
				);

				Transition::step(self.parent)
			}

			Some(b'$') => todo!(),

			// This must be the start of the literal, because the WordContext instance for
			// DoubleQuoted guarantees that the only non-word characters are the closing quote
			// and the dollar.
			Some(_) => Transition::resume(Word::from(self)),

			None => Transition::error(
				self,
				Error {
					error: ErrorKind::UnexpectedEof,
					pos: cursor.pos(),
				}
			)
		}
	}
}


impl<'a> From<Arg<'a>> for DoubleQuoted<'a> {
	fn from(parent: Arg<'a>) -> Self {
		Self {
			parts: Vec::with_capacity(1), // We expect most literals to not be empty.
			errors: Vec::new(),
			parent,
		}
	}
}


impl<'a> WordContext<'a> for DoubleQuoted<'a> {
	fn resume(mut self, value: Vec<u8>) -> Transition<'a> {
		self.parts.push(
			BasicArgument::Literal(value.into_boxed_slice())
		);

		Transition::resume(self)
	}

	fn resume_error(mut self, errors: Vec<InvalidEscapeCode<'a>>) -> Transition<'a> {
		self.errors.extend(errors);
		Transition::resume(self)
	}

	fn is_word(value: u8) -> bool {
		// Comments, single quotes, symbols and whitespace are literals in double quotes.
		value != b'"' && value !=  b'$'
	}

	fn validate_escape(value: u8) -> Option<u8> {
		match value {
			// Syntactical escape sequences:
			b'"' => Some(value), // Escaped quotes.
			b'$' => Some(value), // Escaped dollar.

			// Additional escape sequences:
			b'n' => Some(b'\n'),
			b't' => Some(b'\t'),
			b'\\' => Some(b'\\'),

			// Invalid escape sequence:
			_ => None,
		}
	}
}


impl<'a> From<DoubleQuoted<'a>> for State<'a> {
	fn from(state: DoubleQuoted<'a>) -> State<'a> {
		State::DoubleQuoted(state)
	}
}


#[derive(Debug)]
pub(super) struct Word<'a, C> {
	/// The parsed bytes, if any.
	value: Vec<u8>,
	/// Escape sequence errors, if any.
	errors: Vec<InvalidEscapeCode<'a>>,
	/// The position of the current escape sequence, if any.
	escaping: Option<usize>,
	/// The argument context.
	context: C,
}


impl<'a, C> Word<'a, C>
where
	C: WordContext<'a>,
	State<'a>: From<Self>,
{
	pub fn visit(mut self, cursor: &Cursor<'a>) -> Transition<'a> {
		match (&self, cursor.peek()) {
			// Escaped character.
			(&Self { escaping: Some(escape_offset), .. }, Some(value)) => {
				if let Some(c) = C::validate_escape(value) {
					self.value.push(c);
				} else { // Invalid escape sequence.
					self.errors.push(
						&cursor.slice()[escape_offset ..= cursor.offset()],
					);
				}

				self.escaping = None;
				Transition::step(self)
			}

			// Begin of escape sequence.
			(_, Some(b'\\')) => {
				self.escaping = Some(cursor.offset());
				Transition::step(self)
			}

			// Word character.
			(_, Some(c)) if C::is_word(c) => {
				self.value.push(c);
				Transition::step(self)
			}

			// End of word. Let the context deal with EOF.
			_ => {
				if self.errors.is_empty() {
					self.context.resume(self.value)
				} else {
					self.context.resume_error(self.errors)
				}
			}
		}
	}
}


impl<'a, C: WordContext<'a>> From<C> for Word<'a, C> {
	fn from(context: C) -> Self {
		Self {
			value: Vec::with_capacity(10), // We expect most literals to not be empty.
			errors: Vec::new(),
			escaping: None,
			context,
		}
	}
}


impl<'a> From<Word<'a, Arg<'a>>> for State<'a> {
	fn from(state: Word<'a, Arg<'a>>) -> State<'a> {
		State::UnquotedWord(state)
	}
}


impl<'a> From<Word<'a, SingleQuoted<'a>>> for State<'a> {
	fn from(state: Word<'a, SingleQuoted<'a>>) -> State<'a> {
		State::SingleQuotedWord(state)
	}
}


impl<'a> From<Word<'a, DoubleQuoted<'a>>> for State<'a> {
	fn from(state: Word<'a, DoubleQuoted<'a>>) -> State<'a> {
		State::DoubleQuotedWord(state)
	}
}
