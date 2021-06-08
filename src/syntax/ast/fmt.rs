use std::fmt::Display as _;

use super::{
	lexer::{CommandOperator, Keyword, Operator, TokenKind},
	ArgPart,
	ArgUnit,
	Argument,
	Ast,
	BasicCommand,
	BinaryOp,
	Block,
	Command,
	CommandBlockKind,
	Expr,
	Literal,
	Redirection,
	RedirectionTarget,
	Statement,
	UnaryOp,
};
use crate::{
	fmt::{Display, Indentation},
	symbol,
	syntax::SourcePos,
	term::color
};


pub const ILL_FORMED: color::Fg<color::Red, &'static str> = color::Fg(color::Red, "***ill-formed***");


/// The context for displaying AST nodes.
#[derive(Debug, Copy, Clone)]
pub struct Context<'a> {
	interner: &'a symbol::Interner,
	/// Indentation level. None indicates inline notation.
	indentation: Option<Indentation>,
}


impl<'a> Context<'a> {
	/// Increase the indentation level.
	fn indent(mut self) -> Self {
		self.indentation = self.indentation.map(Indentation::increase);
		self
	}


	/// Set to inlined
	fn inlined(mut self) -> Self {
		self.indentation = None;
		self
	}
}


impl<'a> From<&'a symbol::Interner> for Context<'a> {
	fn from(interner: &'a symbol::Interner) -> Self {
		Self { interner, indentation: Some(Indentation::default()) }
	}
}


impl<'a> Display<'a> for Block {
	type Context = Context<'a>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		sep_by(
			self.0.iter(),
			f,
			|statement, f| {
				if let Some(indent) = context.indentation {
					indent.fmt(f)?;
				} else {
					" ".fmt(f)?;
				}
				statement.fmt(f, context)
			},
			if context.indentation.is_some() { "\n" } else { ";" },
		)
	}
}


impl<'a> Display<'a> for Literal {
	type Context = Context<'a>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Nil => color::Fg(color::Blue, "nil").fmt(f),

			Self::Bool(b) => color::Fg(color::Blue, b).fmt(f),

			Self::Int(i) => i.fmt(f),

			Self::Float(n) => n.fmt(f),

			Self::Byte(c) => write!(f, "'{}'", color::Bold((*c as char).escape_debug())),

			Self::String(s) => write!(
				f,
				"\"{}\"",
				color::Bold(String::from_utf8_lossy(s).escape_debug())
			),

			Self::Array(arr) => {
				let nested = context.indent();

				"[".fmt(f)?;

				sep_by(
					arr.iter(),
					f,
					|item, f| {
						step(f, nested)?;
						item.fmt(f, nested)
					},
					",",
				)?;

				if !arr.is_empty() {
					step(f, context)?;
				}

				"]".fmt(f)
			},

			Self::Dict(dict) => {
				let nested = context.indent();

				"@[".fmt(f)?;

				sep_by(
					dict.iter(),
					f,
					|(k, v), f| {
						step(f, nested)?;
						k.fmt(f, nested.interner)?;
						": ".fmt(f)?;
						v.fmt(f, nested)
					},
					",",
				)?;

				if !dict.is_empty() {
					step(f, context)?;
				}

				"]".fmt(f)
			},

			Self::Function { args, body } => {
				Keyword::Function.fmt(f)?;
				"(".fmt(f)?;

				sep_by(
					args.iter(),
					f,
					|ident, f| ident.fmt(f, context.interner),
					", "
				)?;

				if let Some(indent) = context.indentation {
					")\n".fmt(f)?;
					body.fmt(f, context.indent())?;
					"\n".fmt(f)?;
					indent.fmt(f)?;
				} else {
					") ".fmt(f)?;
				}

				Keyword::End.fmt(f)
			}

			Self::Identifier(identifier) => identifier.fmt(f, context.interner),
		}
	}
}


impl std::fmt::Display for UnaryOp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Minus => Operator::Minus.fmt(f),
			Self::Not => Operator::Not.fmt(f),
		}
	}
}


impl std::fmt::Display for BinaryOp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Plus => Operator::Plus.fmt(f),
			Self::Minus => Operator::Minus.fmt(f),
			Self::Times => Operator::Times.fmt(f),
			Self::Div => Operator::Div.fmt(f),
			Self::Mod => Operator::Mod.fmt(f),
			Self::Equals => Operator::Equals.fmt(f),
			Self::NotEquals => Operator::NotEquals.fmt(f),
			Self::Greater => Operator::Greater.fmt(f),
			Self::GreaterEquals => Operator::GreaterEquals.fmt(f),
			Self::Lower => Operator::Lower.fmt(f),
			Self::LowerEquals => Operator::LowerEquals.fmt(f),
			Self::And => Operator::And.fmt(f),
			Self::Or => Operator::Or.fmt(f),
			Self::Concat => Operator::Concat.fmt(f),
		}
	}
}


impl<'a> Display<'a> for Expr {
	type Context = Context<'a>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::IllFormed => ILL_FORMED.fmt(f),

			Self::Self_ { .. } => Keyword::Self_.fmt(f),

			Self::Identifier { identifier, .. } => identifier.fmt(f, context.interner),

			Self::Literal { literal, .. } => literal.fmt(f, context),

			Self::UnaryOp { op, operand, .. } => {
				write!(f, "({} ", op)?;
				operand.fmt(f, context)?;
				")".fmt(f)
			},

			Self::BinaryOp { left, op, right, .. } => {
				"(".fmt(f)?;
				left.fmt(f, context)?;
				write!(f, " {} ", op)?;
				right.fmt(f, context)?;
				")".fmt(f)
			}

			Self::If { condition, then, otherwise, .. } => {
				let step = if context.indentation.is_some() { "\n" } else { " " };

				Keyword::If.fmt(f)?;
				" ".fmt(f)?;
				condition.fmt(f, context.inlined())?;
				" ".fmt(f)?;
				Keyword::Then.fmt(f)?;
				step.fmt(f)?;

				if !then.0.is_empty() {
					then.fmt(f, context.indent())?;
					step.fmt(f)?;
				}

				if let Some(indent) = context.indentation {
					indent.fmt(f)?;
				}

				if !otherwise.0.is_empty() {
					Keyword::Else.fmt(f)?;
					step.fmt(f)?;
					otherwise.fmt(f, context.indent())?;
					step.fmt(f)?;
				}

				if let Some(indent) = context.indentation {
					indent.fmt(f)?;
				}

				Keyword::End.fmt(f)
			}

			Self::Access { object, field, .. }
			if matches!(field.as_ref(), Self::Literal { literal: Literal::Identifier(..), .. }) => {
				object.fmt(f, context)?;
				".".fmt(f)?;
				field.fmt(f, context.inlined())
			}

			Self::Access { object, field, .. } => {
				object.fmt(f, context)?;
				"[".fmt(f)?;
				field.fmt(f, context.inlined())?;
				"]".fmt(f)
			}

			Self::Call { function, params, .. } => {
				function.fmt(f, context)?;
				"(".fmt(f)?;

				sep_by(
					params.iter(),
					f,
					|param, f| param.fmt(f, context.inlined()),
					", "
				)?;

				")".fmt(f)
			}

			Self::CommandBlock { kind, commands, .. } => {
				match kind {
					CommandBlockKind::Synchronous => "{",
					CommandBlockKind::Asynchronous => "&{",
					CommandBlockKind::Capture => "${",
				}.fmt(f)?;

				let nested = context.indent();

				sep_by(
					commands.iter(),
					f,
					|cmd, f| {
						step(f, nested)?;
						cmd.fmt(f, context.interner)
					},
					if context.indentation.is_some() { ";" } else { ";" },
				)?;

				step(f, context)?;

				"}".fmt(f)
			}
		}
	}
}


impl<'a> Display<'a> for Statement {
	type Context = Context<'a>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::IllFormed => ILL_FORMED.fmt(f),

			Self::Let { identifier, init, .. } => {
				Keyword::Let.fmt(f)?;
				" ".fmt(f)?;
				identifier.fmt(f, context.interner)?;
				" = ".fmt(f)?;
				init.fmt(f, context)
			}

			Self::Assign { left, right, .. } => {
				left.fmt(f, context.inlined())?;
				" = ".fmt(f)?;
				right.fmt(f, context)
			}

			Self::Return { expr, .. } => {
				Keyword::Return.fmt(f)?;
				" ".fmt(f)?;
				expr.fmt(f, context)
			}

			Self::Break { .. } => Keyword::Break.fmt(f),

			Self::While { condition, block, .. } => {
				let step = if context.indentation.is_some() { "\n" } else { " " };

				Keyword::While.fmt(f)?;
				" ".fmt(f)?;
				condition.fmt(f, context.inlined())?;
				" ".fmt(f)?;
				Keyword::Do.fmt(f)?;
				step.fmt(f)?;

				if !block.0.is_empty() {
					block.fmt(f, context.indent())?;
					step.fmt(f)?;
				}

				if let Some(indent) = context.indentation {
					indent.fmt(f)?;
				}

				Keyword::End.fmt(f)
			}

			Self::For { identifier, expr, block, .. } => {
				let step = if context.indentation.is_some() { "\n" } else { " " };

				Keyword::For.fmt(f)?;
				" ".fmt(f)?;
				identifier.fmt(f, context.interner)?;
				" ".fmt(f)?;
				Keyword::In.fmt(f)?;
				" ".fmt(f)?;
				expr.fmt(f, context.inlined())?;
				" ".fmt(f)?;
				Keyword::Do.fmt(f)?;
				step.fmt(f)?;

				if !block.0.is_empty() {
					block.fmt(f, context.indent())?;
					step.fmt(f)?;
				}

				if let Some(indent) = context.indentation {
					indent.fmt(f)?;
				}

				Keyword::End.fmt(f)
			}

			Self::Expr(expr) => expr.fmt(f, context),
		}
	}
}


impl<'a> Display<'a> for ArgUnit {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Literal(lit) => String::from_utf8_lossy(lit).escape_debug().fmt(f),

			Self::Dollar(identifier) => {
				"${".fmt(f)?;
				identifier.fmt(f, context)?;
				"}".fmt(f)
			},
		}
	}
}


impl<'a> Display<'a> for ArgPart {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Unit(unit) => unit.fmt(f, context),
			Self::Home => "~/".fmt(f),
			Self::Range(start, end) => write!(f, "{{{}..{}}}", start, end),
			Self::Collection(items) => {
				"{".fmt(f)?;

				sep_by(
					items.iter(),
					f,
					|item, f| item.fmt(f, context),
					","
				)?;

				"}".fmt(f)
			},
			Self::Star => "*".fmt(f),
			Self::Question => "?".fmt(f),
			Self::CharClass(chars) => write!(f, "[{}]", String::from_utf8_lossy(chars).escape_debug()),
		}
	}
}


impl<'a> Display<'a> for Argument {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		if self.pos == SourcePos::ill_formed() {
			ILL_FORMED.fmt(f)
		} else {
			'"'.fmt(f)?;

			for part in self.parts.iter() {
				part.fmt(f, context)?;
			}

			'"'.fmt(f)
		}
	}
}


impl<'a> Display<'a> for RedirectionTarget {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Fd(fd) => write!(f, ">{}", fd),

			Self::Overwrite(arg) => {
				">".fmt(f)?;
				arg.fmt(f, context)
			}

			Self::Append(arg) => {
				">>".fmt(f)?;
				arg.fmt(f, context)
			},
		}
	}
}


impl<'a> Display<'a> for Redirection {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::IllFormed => ILL_FORMED.fmt(f),

			Self::Output { source, target } => {
				source.fmt(f)?;
				target.fmt(f, context)
			}

			Self::Input { literal: false, source } => {
				"<".fmt(f)?;
				source.fmt(f, context)
			}

			Self::Input { literal: true, source } => {
				"<<".fmt(f)?;
				source.fmt(f, context)
			}
		}
	}
}


impl<'a> Display<'a> for BasicCommand {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		self.command.fmt(f, context)?;

		for arg in self.arguments.iter() {
			" ".fmt(f)?;
			arg.fmt(f, context)?;
		}

		for redirection in self.redirections.iter() {
			" ".fmt(f)?;
			redirection.fmt(f, context)?;
		}

		if self.abort_on_error {
			" ".fmt(f)?;
			CommandOperator::Try.fmt(f)?;
		}

		Ok(())
	}
}


impl<'a> Display<'a> for Command {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		let mut commands = self.0.iter();

		let command = commands.next().expect("empty command");

		command.fmt(f, context)?;

		for command in commands {
			" ".fmt(f)?;
			TokenKind::Pipe.fmt(f, context)?;
			" ".fmt(f)?;
			command.fmt(f, context)?;
		}

		Ok(())
	}
}


impl<'a> Display<'a> for Ast {
	type Context = Context<'a>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		if context.indentation.is_some() {
			writeln!(f, "{} for {}", color::Fg(color::Yellow, "AST"), self.path.display())?;
		}

		self.statements.fmt(f, context)
	}
}


/// Format a sequence of items with a separator.
fn sep_by<'a, T, I, F>(
	mut iter: I,
	f: &mut std::fmt::Formatter,
	mut format: F,
	separator: &str,
) -> std::fmt::Result
where
	I: Iterator<Item = T>,
	F: FnMut(T, &mut std::fmt::Formatter) -> std::fmt::Result,
{
	if let Some(item) = iter.next() {
		format(item, f)?;
	}

	for item in iter {
		separator.fmt(f)?;
		format(item, f)?;
	}

	Ok(())
}


fn step(f: &mut std::fmt::Formatter, ctx: Context) -> std::fmt::Result {
	if let Some(indent) = ctx.indentation {
		"\n".fmt(f)?;
		indent.fmt(f)
	} else {
		" ".fmt(f)
	}
}
