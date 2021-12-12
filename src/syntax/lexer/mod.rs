mod automata;
mod cursor;
mod error;
#[cfg(test)]
mod tests;
mod token;

use crate::symbol;
use automata::Automata;
use super::{Source, SourcePos};
pub use cursor::{Cursor, Checkpoint};
pub use error::{Error, ErrorKind};
pub use token::{
	ArgPart,
	ArgUnit,
	ArgExpansion,
	CommandOperator,
	Keyword,
	Literal,
	Operator,
	Token,
	TokenKind
};


/// The lexer for Hush source code.
#[derive(Debug)]
pub struct Lexer<'a, 'b>(Automata<'a, 'b>);


impl<'a, 'b> Lexer<'a, 'b> {
	pub fn new(cursor: Cursor<'a>, interner: &'b mut symbol::Interner) -> Self {
		Self(Automata::new(cursor, interner))
	}
}


impl<'a, 'b> Iterator for Lexer<'a, 'b> {
	type Item = Result<Token, Error>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}
