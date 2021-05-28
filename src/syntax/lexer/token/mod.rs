mod debug;

use super::SourcePos;
use crate::symbol::Symbol;


/// All keywords in the language, except for operator keywords (and, or, not).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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


/// Literals for non-composite types.
#[derive(Clone, PartialEq)]
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


/// Non-command operators.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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


impl Operator {
	/// Strict equality operators (==, !=).
	pub fn is_equality(&self) -> bool {
		matches!(
			self,
			Self::Equals
				| Self::NotEquals
		)
	}


	/// Non-strict comparison operators (>, >=, <, <=).
	pub fn is_comparison(&self) -> bool {
		matches!(
			self,
			Self::Lower
				| Self::LowerEquals
				| Self::Greater
				| Self::GreaterEquals
		)
	}


	/// Additive arithmetic operators (+, -).
	pub fn is_term(&self) -> bool {
		matches!(
			self,
			Self::Plus
				| Self::Minus
		)
	}


	/// Multiplicative arithmetic operators (*, /, %).
	pub fn is_factor(&self) -> bool {
		matches!(
			self,
			Self::Times
				| Self::Div
				| Self::Mod
		)
	}


	/// Unary operators (-, not)
	pub fn is_unary(&self) -> bool {
		matches!(
			self,
			Self::Not
				| Self::Minus
		)
	}
}


/// The indivisible part of a command argument.
#[derive(Clone, PartialEq)]
pub enum ArgUnit {
	Literal(Box<[u8]>),
	Dollar(Symbol), // $, ${}
}


/// Argument parts may be single, double ou unquoted.
#[derive(Clone, PartialEq)]
pub enum ArgPart {
	Unquoted(ArgUnit),
	SingleQuoted(Box<[u8]>),
	DoubleQuoted(Box<[ArgUnit]>),
}


/// Operators in command blocks.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandOperator {
	OutputRedirection { overwrite: bool }, // >, >>
	InputRedirection { literal: bool },    // <, <<
	Try,                                   // ?
}


/// All possible kinds of token in Hush.
#[derive(Clone, PartialEq)]
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

	// Command block tokens
	Command,        // {
	CaptureCommand, // ${
	AsyncCommand,   // &{
	CloseCommand,   // }

	// A single argument may be composed of many parts.
	Argument(Box<[ArgPart]>),
	CommandOperator(CommandOperator),
	// Semicolons and pipes are not considered operators because they separate different
	// commands, instead of being attributed to a single command.
	Semicolon, // ;
	Pipe,      // |
}


/// A lexical token.
#[derive(Clone)]
pub struct Token {
	pub token: TokenKind,
	pub pos: SourcePos,
}
