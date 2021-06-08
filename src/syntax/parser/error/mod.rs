mod fmt;

use super::{SourcePos, Token, TokenKind};


/// The kind of token the parser was expecting.
#[derive(Debug)]
pub enum Expected {
	Token(TokenKind),
	Message(&'static str),
}


/// A parser error.
#[derive(Debug)]
pub enum Error {
	/// Premature EOF.
	UnexpectedEof,
	/// Unexpected token.
	Unexpected { token: Token, expected: Expected },
	/// Duplicate parameters in function.
	DuplicateParams { pos: SourcePos },
	/// Duplicate keys in dict literal.
	DuplicateKeys { pos: SourcePos },
	/// Command blocks must have at least one command.
	EmptyCommandBlock { pos: SourcePos },
}


impl Error {
	/// Create an error signaling unexpected EOF.
	pub fn unexpected_eof() -> Self {
		Self::UnexpectedEof
	}


	/// Create an error signaling an unexpected token, and what was expected.
	pub fn unexpected(token: Token, expected: TokenKind) -> Self {
		Self::Unexpected { token, expected: Expected::Token(expected) }
	}


	/// Create an error signaling an unexpected token, and a message.
	pub fn unexpected_msg(token: Token, message: &'static str) -> Self {
		Self::Unexpected { token, expected: Expected::Message(message) }
	}


	/// Create an error signaling a function has duplicate parameters.
	pub fn duplicate_params(pos: SourcePos) -> Self {
		Self::DuplicateParams { pos }
	}


	/// Create an error signaling a dict has duplicate keys.
	pub fn duplicate_keys(pos: SourcePos) -> Self {
		Self::DuplicateKeys { pos }
	}


	/// Create an error signaling a command block is empty.
	pub fn empty_command_block(pos: SourcePos) -> Self {
		Self::EmptyCommandBlock { pos }
	}
}


impl std::error::Error for Error {}
