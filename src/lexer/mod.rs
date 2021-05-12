pub mod cursor;
mod automata;
mod error;
mod token;
#[cfg(test)]
mod tests;

use crate::symbol::Interner as SymbolInterner;
use automata::Automata;
use cursor::{Cursor, SourcePos};
use error::{Error, ErrorKind, InvalidLiteral, InvalidEscapeCode};
use token::{Token, TokenKind, Keyword, Literal, Operator};


#[derive(Debug)]
pub struct Lexer<'a>(Automata<'a>);


impl<'a> Lexer<'a> {
	pub fn new(cursor: Cursor<'a>, interner: &'a mut SymbolInterner) -> Self {
		Self(
			Automata::new(cursor, interner)
		)
	}
}


impl<'a> Iterator for Lexer<'a> {
	type Item = Result<Token, Error<'a>>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}
