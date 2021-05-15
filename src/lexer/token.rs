use super::SourcePos;
use crate::symbol::{Symbol, SymbolExt};

use std::fmt::{self, Debug};


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


impl Debug for Keyword {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(
			match self {
				Self::Let => "let",
				Self::If => "if",
				Self::Then => "then",
				Self::Else => "else",
				Self::End => "end",
				Self::For => "for",
				Self::In => "in",
				Self::Do => "do",
				Self::While => "while",
				Self::Function => "function",
				Self::Return => "return",
				Self::Break => "break",
				Self::Self_ => "self",
			}
		)
	}
}


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


impl Debug for Literal {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Nil => write!(f, "nil"),
			Self::True => write!(f, "true"),
			Self::False => write!(f, "false"),
			Self::Int(i) => write!(f, "{}", *i),
			Self::Float(n) => write!(f, "{}", *n),
			Self::Byte(c) => write!(f, "'{}'", *c as char),
			Self::String(s) => write!(f, "\"{}\"", String::from_utf8_lossy(s)),
		}
	}
}


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


impl Debug for Operator {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(
			match self {
				Self::Plus => "+",
				Self::Minus => "-",
				Self::Times => "*",
				Self::Div => "/",
				Self::Mod => "%",
				Self::Equals => "==",
				Self::NotEquals => "!=",
				Self::Greater => ">",
				Self::GreaterEquals => ">=",
				Self::Lower => "<",
				Self::LowerEquals => "<=",
				Self::Not => "not",
				Self::And => "and",
				Self::Or => "or",
				Self::Concat => "++",
				Self::Dot => ".",
				Self::Assign => "=",
			}
		)
	}
}


/// An argument for a command.
#[derive(Clone, PartialEq)]
pub enum BasicArgument {
	Literal(Box<[u8]>),
	Dollar(Symbol), // $, ${}
}


impl Debug for BasicArgument {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Literal(s) => write!(f, "'{}'", String::from_utf8_lossy(s)),
			Self::Dollar(s) => write!(f, "${}", s.to_usize()),
		}
	}
}


/// Command arguments may be single, double ou unquoted.
#[derive(Clone, PartialEq)]
pub enum Argument {
	Unquoted(BasicArgument),
	SingleQuoted(Box<[u8]>),
	DoubleQuoted(Box<[BasicArgument]>),
}


impl Debug for Argument {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Unquoted(arg) => write!(f, "{:?}", arg)?,
			Self::SingleQuoted(s) => write!(f, "'{}'", String::from_utf8_lossy(s))?,
			Self::DoubleQuoted(args) => {
				for arg in args.iter() {
					write!(f, "{:?}", arg)?
				}
			}
		}
		Ok(())
	}
}


// All command operators must be placed after the last argument of a command.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandOperator {
	OutputRedirection { overwrite: bool }, // >, >>
	InputRedirection { literal: bool },    // <, <<
	Try,                                   // ?
}


impl Debug for CommandOperator {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(
			match self {
				Self::OutputRedirection { overwrite: true } => ">>",
				Self::OutputRedirection { overwrite: false } => ">",
				Self::InputRedirection { literal: true } => "<<",
				Self::InputRedirection { literal: false } => "<",
				Self::Try => "?",
			}
		)
	}
}


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
	Argument(Box<[Argument]>),
	CommandOperator(CommandOperator),
	// Semicolons and pipes are not considered operators because they separate different
	// commands, instead of being attribute to a single command.
	Semicolon, // ;
	Pipe,      // |
}


impl Debug for TokenKind {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Identifier(s) => write!(f, "id#{}", s.to_usize())?,
			Self::Keyword(kw) => write!(f, "{:?}", kw)?,
			Self::Operator(op) => write!(f, "{:?}", op)?,
			Self::Literal(lit) => write!(f, "{:?}", lit)?,
			Self::Colon => write!(f, ":")?,
			Self::Comma => write!(f, ",")?,
			Self::OpenParens => write!(f, "(")?,
			Self::CloseParens => write!(f, ")")?,
			Self::OpenBracket => write!(f, "[")?,
			Self::OpenDict => write!(f, "@[")?,
			Self::CloseBracket => write!(f, "]")?,
			Self::Command => write!(f, "{{")?,
			Self::CaptureCommand => write!(f, "${{")?,
			Self::AsyncCommand => write!(f, "&{{")?,
			Self::CloseCommand => write!(f, "}}")?,
			Self::Argument(parts) => for part in parts.iter() {
				write!(f, "{:?}", part)?
			},
			Self::CommandOperator(op) => write!(f, "{:?}", op)?,
			Self::Semicolon => write!(f, ";")?,
			Self::Pipe => write!(f, "|")?,
		}
		Ok(())
	}
}


#[derive(Debug, Clone)]
pub struct Token {
	pub token: TokenKind,
	pub pos: SourcePos,
}
