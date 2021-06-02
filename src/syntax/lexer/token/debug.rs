use std::fmt::{self, Debug};

use super::{ArgPart, ArgUnit, CommandOperator, Keyword, Literal, Operator, Token, TokenKind};
use crate::symbol::SymbolExt;


impl Debug for Keyword {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(match self {
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
		})
	}
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


impl Debug for Operator {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(match self {
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
		})
	}
}


impl Debug for ArgUnit {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Literal(s) => write!(f, "{}", String::from_utf8_lossy(s)),
			Self::Dollar(s) => write!(f, "${{id#{}}}", s.to_usize()),
		}
	}
}


impl Debug for ArgPart {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Unquoted(arg) => write!(f, "{:?}", arg)?,
			Self::SingleQuoted(s) => write!(f, "'{}'", String::from_utf8_lossy(s))?,
			Self::DoubleQuoted(args) => {
				write!(f, "\"")?;

				for arg in args.iter() {
					write!(f, "{:?}", arg)?
				}

				write!(f, "\"")?;
			}
		}
		Ok(())
	}
}


impl Debug for CommandOperator {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(match self {
			Self::Output { append: true } => ">>",
			Self::Output { append: false } => ">",
			Self::Input { literal: true } => "<<",
			Self::Input { literal: false } => "<",
			Self::Try => "?",
		})
	}
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
			Self::Argument(parts) => {
				for part in parts.iter() {
					write!(f, "{:?}", part)?
				}
			}
			Self::CmdOperator(op) => write!(f, "{:?}", op)?,
			Self::Semicolon => write!(f, ";")?,
			Self::Pipe => write!(f, "|")?,
		}
		Ok(())
	}
}


impl Debug for Token {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if f.alternate() {
			writeln!(f, "{}: {:?}", self.pos, self.token)
		} else {
			write!(f, "{:?}", self.token)
		}
	}
}
