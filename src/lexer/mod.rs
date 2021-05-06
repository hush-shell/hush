pub mod command;

use crate::symbol::{self, Symbol};

use std::fmt::{self, Display};


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourcePos {
	pub line: u32,
	pub column: u32,
}


impl Display for SourcePos {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "line {}, column {}", self.line, self.column)
	}
}


#[derive(Debug, Clone, PartialEq)]
pub struct Token<Kind> {
	kind: Kind,
	pos: SourcePos,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
	Let,
	If,
	Then,
	Else,
	End,
	For,
	In,
	Do,
	While,
	Function,
	Return,
	Break,
	Self_,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
	Nil,
	True,
	False,
	Int(i64),
	Float(f64),
	Char(char),
	// String literals are not interned because they probably won't be repeated very often.
	String(Box<str>),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
	Plus,  // +
	Minus, // -
	Times, // *
	Div,   // /
	Mod,   // %

	Equal,        // ==
	NotEqual,     // !=
	Greater,      // >
	GreaterEqual, // >=
	Lower,        // <
	LowerEqual,   // <=

	Not, // not
	And, // and
	Or,  // or

	Concat, // ++
	Dot,    // .

	Assign, // =
}


#[derive(Debug)]
pub enum TokenKind<'a> {
	Identifier(Symbol),
	Keyword(Keyword),
	Literal(Literal),

	OpenBracket,  // [
	OpenDict,     // @[
	CloseBracket, // ]

	Colon, // :
	Comma, // ,

	OpenParens,  // (
	CloseParens, // )

	// Commands require a different lexer mode:
	Command(command::Lexer<'a>),        // {}
	CaptureCommand(command::Lexer<'a>), // ${}
	AsyncCommand(command::Lexer<'a>),   // &{}
}


pub trait Scanner<'a> {
	type Token: 'a;

	fn next(&'a mut self) -> Option<Self::Token>;
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

	fn peek(&self) -> Option<char> {
		self.input.chars().next()
	}


	fn peek_next(&self) -> Option<char> {
		self.input.chars().nth(1)
	}


	fn advance(&mut self) -> Option<char> {
		let mut chars = self.input.chars();
		let result = chars.next();

		self.input = chars.as_str();
		if let Some('\n') = result {
			self.pos.line += 1;
			self.pos.column = 0;
		} else {
			self.pos.column += 1;
		}

		result
	}

	fn match_next(&mut self, expected: char) -> bool {
		match self.peek() {
			Some(next) if next == expected => {
				self.advance();
				true
			},
			_ => false,
		}
	}
}


impl<'a> Scanner<'a> for Lexer<'a> {
	type Token = Token<TokenKind<'a>>;

	fn next(&'a mut self) -> Option<Self::Token> {
		todo!()
	}
}
