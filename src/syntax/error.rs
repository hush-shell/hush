use std::fmt::{self, Display};

use super::{lexer, parser};


/// Syntax error.
#[derive(Debug)]
pub enum Error {
	Lexer(lexer::Error),
	Parser(parser::Error),
}


impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Lexer(error) => write!(f, "{}", error),
			Self::Parser(error) => write!(f, "{}", error),
		}
	}
}


impl std::error::Error for Error {}


#[derive(Debug)]
pub struct DisplayErrors<'a>(pub &'a [Error]);


impl<'a> Display for DisplayErrors<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for error in self.0 {
			writeln!(f, "{}", error)?;
		}

		Ok(())
	}
}
