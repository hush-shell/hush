mod fmt;

use super::{lexer, parser, AnalysisDisplayContext};


/// Syntax error.
#[derive(Debug)]
pub enum Error {
	Lexer(lexer::Error),
	Parser(parser::Error),
}


impl std::error::Error for Error {}


/// Syntax errors.
#[derive(Debug)]
pub struct Errors(pub Box<[Error]>);


impl Errors {
	/// Check if there are any errors.
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}
