use std::fmt::{self, Debug, Display};

use super::SourcePos;


/// The kind of lexical error.
pub enum ErrorKind<'a> {
	/// Unexpected end of file.
	UnexpectedEof,
	/// Unexpected character.
	Unexpected(u8),
	/// Empty byte literal ('').
	EmptyByteLiteral,
	/// Invalid escape sequence in byte literal, string literal, or argument literal.
	InvalidEscapeSequence(&'a [u8]),
	/// Invalid number literal, both integer and floating point.
	InvalidNumber(&'a [u8]),
	/// Invalid identifier, only possible in dollar braces (${}).
	InvalidIdentifier(&'a [u8]),
}


impl<'a> Debug for ErrorKind<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self) // Use the display instance for debugging.
	}
}


impl<'a> Display for ErrorKind<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			ErrorKind::UnexpectedEof => write!(f, "unexpected end of file")?,

			ErrorKind::Unexpected(value) => write!(f, "unexpected '{}'", *value as char)?,

			ErrorKind::EmptyByteLiteral => write!(f, "empty char literal")?,

			ErrorKind::InvalidEscapeSequence(sequence) => {
				write!(
					f,
					"invalid escape sequence: {}",
					String::from_utf8_lossy(sequence)
				)?;
			}

			ErrorKind::InvalidNumber(number) => {
				write!(f, "invalid number: {}", String::from_utf8_lossy(number))?;
			}

			ErrorKind::InvalidIdentifier(ident) => {
				write!(f, "invalid identifier: {}", String::from_utf8_lossy(ident))?;
			}
		};

		Ok(())
	}
}


/// A lexical error.
#[derive(Debug)]
pub struct Error<'a> {
	pub error: ErrorKind<'a>,
	pub pos: SourcePos,
}


impl<'a> Display for Error<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Error at {}: {}.", self.pos, self.error)
	}
}


impl<'a> Error<'a> {
	pub fn unexpected_eof(pos: SourcePos) -> Self {
		Self { error: ErrorKind::UnexpectedEof, pos }
	}

	pub fn unexpected(input: u8, pos: SourcePos) -> Self {
		Self { error: ErrorKind::Unexpected(input), pos }
	}

	pub fn empty_byte_literal(pos: SourcePos) -> Self {
		Self { error: ErrorKind::EmptyByteLiteral, pos }
	}

	pub fn invalid_escape_sequence(sequence: &'a [u8], pos: SourcePos) -> Self {
		Self {
			error: ErrorKind::InvalidEscapeSequence(sequence),
			pos,
		}
	}

	pub fn invalid_number(number: &'a [u8], pos: SourcePos) -> Self {
		Self { error: ErrorKind::InvalidNumber(number), pos }
	}

	pub fn invalid_identifier(ident: &'a [u8], pos: SourcePos) -> Self {
		Self { error: ErrorKind::InvalidIdentifier(ident), pos }
	}
}
