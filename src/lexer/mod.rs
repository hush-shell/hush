mod automata;
pub mod cursor;
mod error;
mod token;
#[cfg(test)]
mod tests;

use crate::{
	source::Pos as SourcePos,
	symbol::Interner as SymbolInterner,
};
use automata::Automata;
use cursor::Cursor;
use error::{Error, ErrorKind};
use token::{ArgPart, ArgUnit, Keyword, Literal, CommandOperator, Operator, Token, TokenKind};


/// The lexer for Hush source code.
#[derive(Debug)]
pub struct Lexer<'a, 'b>(Automata<'a, 'b>);


impl<'a, 'b> Lexer<'a, 'b> {
	pub fn new(cursor: Cursor<'a>, interner: &'b mut SymbolInterner) -> Self {
		Self(Automata::new(cursor, interner))
	}
}


impl<'a, 'b> Iterator for Lexer<'a, 'b> {
	type Item = Result<Token, Error<'a>>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}
