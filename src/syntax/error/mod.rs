mod fmt;

use super::{lexer, parser};


/// Syntax error.
#[derive(Debug)]
pub enum Error {
	Lexer(lexer::Error),
	Parser(parser::Error),
}


impl std::error::Error for Error {}
