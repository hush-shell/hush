use std::fmt::Display as _;

use super::{
	ArgPart,
	ArgExpansion,
	ArgUnit,
	CommandOperator,
	Keyword,
	Literal,
	Operator,
	Token,
	TokenKind
};
use crate::{
	fmt::{self, Display},
	symbol,
	term::color,
};


impl std::fmt::Display for Keyword {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		color
			::Fg(
				color::Blue,
				match self {
					Self::Let => "let",
					Self::If => "if",
					Self::Then => "then",
					Self::Else => "else",
					Self::ElseIf => "elseif",
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
			.fmt(f)
	}
}


impl std::fmt::Display for Literal {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Nil => color::Fg(color::Blue, "nil").fmt(f),
			Self::True => color::Fg(color::Blue, "true").fmt(f),
			Self::False => color::Fg(color::Blue, "false").fmt(f),
			Self::Int(i) => i.fmt(f),
			Self::Float(n) => n.fmt(f),
			Self::Byte(c) => write!(f, "'{}'", color::Bold((*c as char).escape_debug())),
			Self::String(s) => write!(
				f,
				"\"{}\"",
				color::Bold(String::from_utf8_lossy(s).escape_debug())
			),
		}
	}
}


impl std::fmt::Display for Operator {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Plus => color::Fg(color::Yellow, "+").fmt(f),
			Self::Minus => color::Fg(color::Yellow, "-").fmt(f),
			Self::Times => color::Fg(color::Yellow, "*").fmt(f),
			Self::Div => color::Fg(color::Yellow, "/").fmt(f),
			Self::Mod => color::Fg(color::Yellow, "%").fmt(f),
			Self::Equals => color::Fg(color::Yellow, "==").fmt(f),
			Self::NotEquals => color::Fg(color::Yellow, "!=").fmt(f),
			Self::Greater => color::Fg(color::Yellow, ">").fmt(f),
			Self::GreaterEquals => color::Fg(color::Yellow, ">=").fmt(f),
			Self::Lower => color::Fg(color::Yellow, "<").fmt(f),
			Self::LowerEquals => color::Fg(color::Yellow, "<=").fmt(f),
			Self::Not => color::Fg(color::Blue, "not").fmt(f),
			Self::And => color::Fg(color::Blue, "and").fmt(f),
			Self::Or => color::Fg(color::Blue, "or").fmt(f),
			Self::Concat => color::Fg(color::Yellow, "++").fmt(f),
			Self::Dot => color::Fg(color::Yellow, ".").fmt(f),
			Self::Assign => "=".fmt(f),
			Self::Try => color::Fg(color::Yellow, "?").fmt(f),
		}
	}
}


impl<'a> Display<'a> for ArgUnit {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Literal(s) => String::from_utf8_lossy(s).escape_debug().fmt(f),
			Self::Dollar { symbol, .. } => {
				"${{".fmt(f)?;
				symbol.fmt(f, context)?;
				"}}".fmt(f)
			}
		}
	}
}


impl<'a> Display<'a> for ArgExpansion {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Home => color::Fg(color::Yellow, "~/").fmt(f),
			Self::Range(start, end) => {
				color::Fg(color::Yellow, "{").fmt(f)?;
				start.fmt(f)?;
				color::Fg(color::Yellow, "..").fmt(f)?;
				end.fmt(f)?;
				color::Fg(color::Yellow, "}").fmt(f)
			},
			Self::Collection(items) => {
				color::Fg(color::Yellow, "{").fmt(f)?;

				fmt::sep_by(
					items.iter(),
					f,
					|item, f| item.fmt(f, context),
					color::Fg(color::Yellow, ",")
				)?;

				color::Fg(color::Yellow, "}").fmt(f)
			},

			Self::Star => color::Fg(color::Yellow, "*").fmt(f),
			Self::Percent => color::Fg(color::Yellow, "%").fmt(f),
			Self::CharClass(chars) => {
				color::Fg(color::Yellow, "[").fmt(f)?;

				color
					::Fg(
						color::Yellow,
						String::from_utf8_lossy(chars).escape_debug()
					)
					.fmt(f)?;

				color::Fg(color::Yellow, "]").fmt(f)
			},
		}
	}
}


impl<'a> Display<'a> for ArgPart {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Unquoted(arg) => arg.fmt(f, context),
			Self::SingleQuoted(s) => write!(f, "'{}'", String::from_utf8_lossy(s).escape_debug()),
			Self::DoubleQuoted(args) => {
				'"'.fmt(f)?;

				for arg in args.iter() {
					arg.fmt(f, context)?;
				}

				'"'.fmt(f)
			},
			Self::Expansion(expansion) => expansion.fmt(f, context),
			Self::EnvAssign => color::Fg(color::Yellow, "=").fmt(f),
		}
	}
}


impl std::fmt::Display for CommandOperator {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		color
			::Fg(
				color::Yellow,
				match self {
					Self::Output { append: true } => ">>",
					Self::Output { append: false } => ">",
					Self::Input { literal: true } => "<<",
					Self::Input { literal: false } => "<",
					Self::Try => "?",
				}
			)
			.fmt(f)
	}
}


impl<'a> Display<'a> for TokenKind {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Identifier(s) => s.fmt(f, context),
			Self::Keyword(kw) => kw.fmt(f),
			Self::Operator(op) => op.fmt(f),
			Self::Literal(lit) => lit.fmt(f),
			Self::Colon => ":".fmt(f),
			Self::Comma => ",".fmt(f),
			Self::OpenParens => "(".fmt(f),
			Self::CloseParens => ")".fmt(f),
			Self::OpenBracket => "[".fmt(f),
			Self::OpenDict => "@[".fmt(f),
			Self::CloseBracket => "]".fmt(f),
			Self::Command => "{".fmt(f),
			Self::CaptureCommand => "${".fmt(f),
			Self::AsyncCommand => "&{".fmt(f),
			Self::CloseCommand => "}".fmt(f),
			Self::Argument(parts) => {
				for part in parts.iter() {
					part.fmt(f, context)?
				}
				Ok(())
			}
			Self::CmdOperator(op) => op.fmt(f),
			Self::Semicolon => ";".fmt(f),
			Self::Pipe => color::Fg(color::Yellow, "|").fmt(f),
		}
	}
}


impl<'a> Display<'a> for Token {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		write!(f, "{}:\t", fmt::Show(self.pos, context))?;
		self.kind.fmt(f, context)
	}
}
