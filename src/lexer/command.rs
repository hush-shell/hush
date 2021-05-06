use super::{Scanner, SourcePos, Token};
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
}


#[derive(Debug)]
pub struct Lexer<'a> {
	input: &'a str,
	symbol_interner: &'a mut symbol::Interner,
	pos: SourcePos,
}


impl<'a> Lexer<'a> {
	pub fn new(
		input: &'a str,
		symbol_interner: &'a mut symbol::Interner,
		pos: SourcePos,
	) -> Self {
		Self {
			input,
			symbol_interner,
			pos,
		}
	}
}


impl<'a> Scanner<'a> for Lexer<'a> {
	type Token = Token<TokenKind>;

	fn next(&'a mut self) -> Option<Self::Token> {
		// Here, the lexer should return EOF (None) when the close command token is consumed.
		todo!()
	}
}
