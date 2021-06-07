use std::fmt::{self, Display};

use super::{SourcePos, Token, TokenKind};


/// The kind of token the parser was expecting.
#[derive(Debug)]
pub enum Expected {
	Token(TokenKind),
	Message(&'static str),
}


impl Display for Expected {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Token(token) => write!(f, "'{:?}'", token),
			Self::Message(msg) => write!(f, "{}", msg),
		}
	}
}


/// A parser error.
#[derive(Debug)]
pub enum Error {
	/// Premature EOF.
	UnexpectedEof,
	/// Unexpected token.
	Unexpected { token: Token, expected: Expected },
	/// Duplicate keys in dict literal.
	DuplicateKeys { pos: SourcePos },
}


impl Error {
	pub fn unexpected_eof() -> Self {
		Self::UnexpectedEof
	}


	pub fn unexpected(token: Token, expected: TokenKind) -> Self {
		Self::Unexpected { token, expected: Expected::Token(expected) }
	}


	pub fn unexpected_msg(token: Token, message: &'static str) -> Self {
		Self::Unexpected { token, expected: Expected::Message(message) }
	}


	pub fn duplicate_keys(pos: SourcePos) -> Self {
		Self::DuplicateKeys { pos }
	}
}


impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::UnexpectedEof => write!(f, "unexpected end of file"),
			Self::Unexpected { token: Token { token, pos }, expected } => {
				write!(
					f,
					"{} - unexpected '{:?}', expected {}",
					pos, token, expected
				)
			},
			Self::DuplicateKeys { pos } => {
				write!(f, "{} - duplicate keys in dict literal", pos)
			}
		}
	}
}


impl std::error::Error for Error {}
