mod fmt;

use super::SourcePos;
use crate::symbol::Symbol;


/// All keywords in the language, except for operator keywords (and, or, not).
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


/// Literals for non-composite types.
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


/// Non-command operators.
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


impl Operator {
	/// Strict equality operators (==, !=).
	pub fn is_equality(&self) -> bool {
		matches!(self, Self::Equals | Self::NotEquals)
	}


	/// Non-strict comparison operators (>, >=, <, <=).
	pub fn is_comparison(&self) -> bool {
		matches!(
			self,
			Self::Lower | Self::LowerEquals | Self::Greater | Self::GreaterEquals
		)
	}


	/// Additive arithmetic operators (+, -).
	pub fn is_term(&self) -> bool {
		matches!(self, Self::Plus | Self::Minus)
	}


	/// Multiplicative arithmetic operators (*, /, %).
	pub fn is_factor(&self) -> bool {
		matches!(self, Self::Times | Self::Div | Self::Mod)
	}


	/// Unary operators (-, not)
	pub fn is_unary(&self) -> bool {
		matches!(self, Self::Not | Self::Minus)
	}
}


/// The indivisible part of a command argument.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgUnit {
	Literal(Box<[u8]>),
	Dollar(Symbol), // $, ${}
}


impl ArgUnit {
	/// Check if the unit is a literal composed only of digits.
	pub fn is_number(&self) -> bool {
		matches!(
			self,
			Self::Literal(lit) if lit.iter().all(u8::is_ascii_digit)
		)
	}
}


/// Argument parts may be single, double ou unquoted.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgPart {
	Unquoted(ArgUnit),
	SingleQuoted(Box<[u8]>),
	DoubleQuoted(Box<[ArgUnit]>),
}


impl ArgPart {
	/// Check if the part is a unquoted literal composed only of digits.
	pub fn is_unquoted_number(&self) -> bool {
		matches!(
			self,
			Self::Unquoted(unit) if unit.is_number()
		)
	}
}


/// Operators in command blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandOperator {
	Output { append: bool }, // >, >>
	Input { literal: bool }, // <, <<
	Try,                     // ?
}


impl CommandOperator {
	/// Check if the operator is a input or output redirection.
	pub fn is_redirection(&self) -> bool {
		matches!(
			self,
			Self::Output { .. } | Self::Input { .. }
		)
	}
}


/// All possible kinds of token in Hush.
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

	// Command block tokens
	Command,        // {
	AsyncCommand,   // &{
	CaptureCommand, // ${
	CloseCommand,   // }

	// A single argument may be composed of many parts.
	Argument(Box<[ArgPart]>),
	CmdOperator(CommandOperator),
	// Semicolons and pipes are not considered operators because they separate different
	// commands, instead of being attributed to a single command.
	Semicolon, // ;
	Pipe,      // |
}


impl TokenKind {
	/// Check if the token terminates a statement block.
	/// Currently, only the END and ELSE keywords do that.
	pub fn is_block_terminator(&self) -> bool {
		matches!(
			self,
			TokenKind::Keyword(Keyword::End) | TokenKind::Keyword(Keyword::Else)
		)
	}


	/// Check if the token terminates a basic command.
	/// Currently, the semicolon, the pipe and the close bracket tokens do that.
	pub fn is_basic_command_terminator(&self) -> bool {
		matches!(
			self,
			TokenKind::Semicolon | TokenKind::Pipe | TokenKind::CloseCommand
		)
	}
}


/// A lexical token.
#[derive(Debug, Clone)]
pub struct Token {
	pub token: TokenKind,
	pub pos: SourcePos,
}
