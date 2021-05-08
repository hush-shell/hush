use super::{
	cursor::{Cursor, SourcePos},
	Token,
};
use crate::symbol::{self, Symbol};


#[derive(Debug, Clone, PartialEq)]
pub enum BasicArgument {
	Literal(Box<str>),
	Dollar(Symbol), // $, ${}
}


#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
	Unquoted(BasicArgument),
	SingleQuoted(Box<str>),
	DoubleQuoted(Box<[BasicArgument]>),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
	OutputRedirection { overwrite: bool }, // >, >>
	InputRedirection { literal: bool },    // <, <<
	Pipe,                                  // |
	Try,                                   // ?
}


#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
	Word(Box<[Argument]>),
	Operator(Operator),
	Semicolon, // ;

	Unexpected(char), // Lex error.
}


#[derive(Debug)]
pub struct Lexer;


impl Lexer {}


// impl<'a> Scanner<'a> for Lexer<'a> {
// 	type Token = Token<TokenKind>;

// 	fn next(&'a mut self, cursor: &'a mut Cursor) -> Option<Self::Token> {
// 		// Here, the lexer should return EOF (None) when the close command token is consumed.
// 		todo!()
// 	}
// }
