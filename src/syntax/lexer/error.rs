use std::fmt::{self, Debug, Display};

use super::SourcePos;


/// The kind of lexical error.
pub enum ErrorKind {
	/// Unexpected end of file.
	UnexpectedEof,
	/// Unexpected character.
	Unexpected(u8),
	/// Empty byte literal ('').
	EmptyByteLiteral,
	/// Invalid escape sequence in byte literal, string literal, or argument literal.
	InvalidEscapeSequence(Box<[u8]>),
	/// Invalid number literal, both integer and floating point.
	InvalidNumber(Box<[u8]>),
	/// Invalid identifier, only possible in dollar braces (${}).
	InvalidIdentifier(Box<[u8]>),
}


impl Debug for ErrorKind {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self) // Use the display instance for debugging.
	}
}


impl Display for ErrorKind {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::UnexpectedEof => write!(f, "unexpected end of file")?,

			Self::Unexpected(value) => write!(f, "unexpected '{}'", *value as char)?,

			Self::EmptyByteLiteral => write!(f, "empty char literal")?,

			Self::InvalidEscapeSequence(sequence) => {
				write!(
					f,
					"invalid escape sequence: {}",
					String::from_utf8_lossy(sequence)
				)?;
			}

			Self::InvalidNumber(number) => {
				write!(f, "invalid number: {}", String::from_utf8_lossy(number))?;
			}

			Self::InvalidIdentifier(ident) => {
				write!(f, "invalid identifier: {}", String::from_utf8_lossy(ident))?;
			}
		};

		Ok(())
	}
}


/// A lexical error.
#[derive(Debug)]
pub struct Error {
	pub error: ErrorKind,
	pub pos: SourcePos,
}


impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Error at {}: {}.", self.pos, self.error)
	}
}


impl std::error::Error for Error {}


impl Error {
	pub fn unexpected_eof(pos: SourcePos) -> Self {
		Self { error: ErrorKind::UnexpectedEof, pos }
	}

	pub fn unexpected(input: u8, pos: SourcePos) -> Self {
		Self { error: ErrorKind::Unexpected(input), pos }
	}

	pub fn empty_byte_literal(pos: SourcePos) -> Self {
		Self { error: ErrorKind::EmptyByteLiteral, pos }
	}

	pub fn invalid_escape_sequence(sequence: &[u8], pos: SourcePos) -> Self {
		Self {
			error: ErrorKind::InvalidEscapeSequence(sequence.into()),
			pos,
		}
	}

	pub fn invalid_number(number: &[u8], pos: SourcePos) -> Self {
		Self {
			error: ErrorKind::InvalidNumber(number.into()),
			pos,
		}
	}

	pub fn invalid_identifier(ident: &[u8], pos: SourcePos) -> Self {
		Self {
			error: ErrorKind::InvalidIdentifier(ident.into()),
			pos,
		}
	}
}
