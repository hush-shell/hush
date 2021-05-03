use super::Symbol;


#[derive(Debug, Clone, PartialEq)]
pub enum BasicArgument<'a> {
	Literal(Box<[u8]>),
	Dollar(Symbol<'a>), // $, ${}
}


#[derive(Debug, Clone, PartialEq)]
pub enum Argument<'a> {
	Unquoted(BasicArgument<'a>),
	SingleQuoted(Box<[u8]>),
	DoubleQuoted(Box<[BasicArgument<'a>]>),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
	OutputRedirection { overwrite: bool }, // >, >>
	InputRedirection { literal: bool },    // <, <<
	Pipe,                                  // |
	Try,                                   // ?
}


#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
	Word(Box<[Argument<'a>]>),
	Operator(Operator),
	Semicolon,
}


// Here, the lexer should return EOF when the close command token is consumed.
#[derive(Debug, Clone)]
pub struct Lexer;
