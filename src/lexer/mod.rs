pub mod command;


/// Symbols are interned in the symbols table.
type Symbol<'a> = &'a [u8];


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
	Char(u8),
	String(Box<[u8]>), // TODO: should we intern string literals?
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


#[derive(Debug, Clone)]
pub enum Token<'a> {
	Identifier(Symbol<'a>),
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
	Command(command::Lexer),        // {}
	CaptureCommand(command::Lexer), // ${}
	AsyncCommand(command::Lexer),   // &{}
}


#[derive(Debug, Clone)]
pub struct Lexer;
