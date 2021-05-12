use crate::symbol::Symbol;
use super::SourcePos;


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
	Byte(u8),
	// String literals are not interned because they probably won't be repeated very often.
	String(Box<[u8]>),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
	Plus,  // +
	Minus, // -
	Times, // *
	Div,   // /
	Mod,   // %

	Equals,        // ==
	NotEquals,     // !=
	Greater,       // >
	GreaterEquals, // >=
	Lower,         // <
	LowerEquals,   // <=

	Not, // not
	And, // and
	Or,  // or

	Concat, // ++
	Dot,    // .

	Assign, // =
}


#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
	Identifier(Symbol),
	Keyword(Keyword),
	Operator(Operator),
	Literal(Literal),

	Colon, // :
	Comma, // ,

	OpenParens,  // (
	CloseParens, // )

	OpenBracket,  // [
	OpenDict,     // @[
	CloseBracket, // ]

	// Commands require a different lexer mode. When any of these tokens are produced, a
	// command lexer should be used to consume all command tokens, including the close block
	// token.
	Command,        // {}
	CaptureCommand, // ${}
	AsyncCommand,   // &{}
}


#[derive(Debug, Clone)]
pub struct Token {
	pub token: TokenKind,
	pub pos: SourcePos,
}
