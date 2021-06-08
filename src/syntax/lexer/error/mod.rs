mod fmt;

use super::SourcePos;


/// The kind of lexical error.
#[derive(Debug)]
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


/// A lexical error.
#[derive(Debug)]
pub struct Error {
	pub error: ErrorKind,
	pub pos: SourcePos,
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
