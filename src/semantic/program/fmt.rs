use std::fmt::Display as _;

use super::{
	command,
	lexer::{CommandOperator, Keyword, Operator},
	mem,
	ArgPart,
	ArgUnit,
	Argument,
	Program,
	BasicCommand,
	BinaryOp,
	Block,
	Command,
	CommandBlock,
	CommandBlockKind,
	Expr,
	Literal,
	Lvalue,
	Redirection,
	RedirectionTarget,
	Statement,
	UnaryOp,
};
use crate::{
	fmt::{self, Display, Indentation},
	symbol,
	term::color
};


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
		fmt::sep_by(
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

				fmt::sep_by(
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

				fmt::sep_by(
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

			Self::Function { params, frame_info, body } => {
				let step = if context.indentation.is_some() { "\n" } else { " " };

				Keyword::Function.fmt(f)?;
				"(".fmt(f)?;

				params.fmt(f)?;

				")".fmt(f)?;

				if context.indentation.is_some() {
					"\n".fmt(f)?;
				}

				frame_info.fmt(f, context.indent().indentation)?;

				step.fmt(f)?;

				body.fmt(f, context.indent())?;

				self::step(f, context)?;

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
			Self::Try => Operator::Try.fmt(f),
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
			Self::Identifier { slot_ix, .. } => slot_ix.fmt(f),

			Self::Literal { literal, .. } => literal.fmt(f, context),

			Self::UnaryOp { op, operand, .. } => {
				let postfix = op.is_postfix();

				"(".fmt(f)?;

				if !postfix {
					write!(f, "{} ", op)?;
				}

				operand.fmt(f, context.inlined())?;

				if postfix {
					write!(f, " {}", op)?;
				}

				")".fmt(f)
			},

			Self::BinaryOp { left, op, right, .. } => {
				"(".fmt(f)?;
				left.fmt(f, context.inlined())?;
				write!(f, " {} ", op)?;
				right.fmt(f, context.inlined())?;
				")".fmt(f)
			}

			Self::If { condition, then, otherwise, .. } => {
				let step = if context.indentation.is_some() { "\n" } else { " " };

				Keyword::If.fmt(f)?;
				" ".fmt(f)?;
				condition.fmt(f, context.inlined())?;
				" ".fmt(f)?;
				Keyword::Then.fmt(f)?;
				if context.indentation.is_some() {
					"\n".fmt(f)?;
				}

				if !then.0.is_empty() {
					then.fmt(f, context.indent())?;
					step.fmt(f)?;
				}

				if let Some(indent) = context.indentation {
					indent.fmt(f)?;
				}

				if !otherwise.0.is_empty() {
					Keyword::Else.fmt(f)?;
					if context.indentation.is_some() {
						"\n".fmt(f)?;
					}

					otherwise.fmt(f, context.indent())?;
					step.fmt(f)?;

					if let Some(indent) = context.indentation {
						indent.fmt(f)?;
					}
				}

				Keyword::End.fmt(f)
			}

			Self::Access { object, field, .. }
			if matches!(field.as_ref(), Self::Literal { literal: Literal::Identifier(..), .. }) => {
				object.fmt(f, context.inlined())?;
				".".fmt(f)?;
				field.fmt(f, context.inlined())
			}

			Self::Access { object, field, .. } => {
				object.fmt(f, context.inlined())?;
				"[".fmt(f)?;
				field.fmt(f, context.inlined())?;
				"]".fmt(f)
			}

			Self::Call { function, args, .. } => {
				function.fmt(f, context.inlined())?;
				"(".fmt(f)?;

				fmt::sep_by(
					args.iter(),
					f,
					|param, f| param.fmt(f, context.inlined()),
					", "
				)?;

				")".fmt(f)
			}

			Self::CommandBlock { block, .. } => block.fmt(f, context),
		}
	}
}


impl<'a> Display<'a> for Lvalue {
	type Context = Context<'a>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Identifier { slot_ix, .. } => slot_ix.fmt(f),

			Self::Access { object, field, .. } => {
				object.fmt(f, context.inlined())?;
				"[".fmt(f)?;
				field.fmt(f, context.inlined())?;
				"]".fmt(f)
			}
		}
	}
}


impl<'a> Display<'a> for Statement {
	type Context = Context<'a>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Assign { left, right } => {
				left.fmt(f, context.inlined())?;
				" = ".fmt(f)?;
				right.fmt(f, context)
			}

			Self::Return { expr } => {
				Keyword::Return.fmt(f)?;
				" ".fmt(f)?;
				expr.fmt(f, context)
			}

			Self::Break => Keyword::Break.fmt(f),

			Self::While { condition, block } => {
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

			Self::For { slot_ix, expr, block } => {
				let step = if context.indentation.is_some() { "\n" } else { " " };

				Keyword::For.fmt(f)?;
				" ".fmt(f)?;
				slot_ix.fmt(f)?;
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


impl std::fmt::Display for ArgUnit {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Literal(lit) => String::from_utf8_lossy(lit).escape_debug().fmt(f),

			Self::Dollar { slot_ix, .. } => {
				"${".fmt(f)?;
				slot_ix.fmt(f)?;
				"}".fmt(f)
			},
		}
	}
}


impl std::fmt::Display for ArgPart {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Unit(unit) => unit.fmt(f),
			Self::Home => "~/".fmt(f),
			Self::Range(start, end) => write!(f, "{{{}..{}}}", start, end),
			Self::Collection(items) => {
				"{".fmt(f)?;

				fmt::sep_by(
					items.iter(),
					f,
					|item, f| item.fmt(f),
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


impl std::fmt::Display for Argument {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		'"'.fmt(f)?;

		for part in self.parts.iter() {
			part.fmt(f)?;
		}

		'"'.fmt(f)
	}
}


impl std::fmt::Display for RedirectionTarget {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Fd(fd) => write!(f, ">{}", fd),

			Self::Overwrite(arg) => {
				">".fmt(f)?;
				arg.fmt(f)
			}

			Self::Append(arg) => {
				">>".fmt(f)?;
				arg.fmt(f)
			},
		}
	}
}


impl std::fmt::Display for Redirection {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Output { source, target } => {
				source.fmt(f)?;
				target.fmt(f)
			}

			Self::Input { literal: false, source } => {
				"<".fmt(f)?;
				source.fmt(f)
			}

			Self::Input { literal: true, source } => {
				"<<".fmt(f)?;
				source.fmt(f)
			}
		}
	}
}


impl std::fmt::Display for command::Builtin {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let command = match self {
			command::Builtin::Alias => "alias",
			command::Builtin::Cd => "cd",
		};

		color::Fg(color::Green, command).fmt(f)
	}
}


impl std::fmt::Display for command::InvalidBuiltin {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		"invalid built-in command".fmt(f)
	}
}


impl std::fmt::Display for BasicCommand {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.program.fmt(f)?;

		for arg in self.arguments.iter() {
			" ".fmt(f)?;
			arg.fmt(f)?;
		}

		for redirection in self.redirections.iter() {
			" ".fmt(f)?;
			redirection.fmt(f)?;
		}

		if self.abort_on_error {
			" ".fmt(f)?;
			CommandOperator::Try.fmt(f)?;
		}

		Ok(())
	}
}


impl std::fmt::Display for Command {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Command::Builtin { program, arguments, abort_on_error, .. } => {
				program.fmt(f)?;

				for arg in arguments.iter() {
					" ".fmt(f)?;
					arg.fmt(f)?;
				}

				if *abort_on_error {
					" ".fmt(f)?;
					CommandOperator::Try.fmt(f)?;
				}
			},

			Command::External { head, tail } => {
				head.fmt(f)?;

				for command in tail.iter() {
					" ".fmt(f)?;
					color::Fg(color::Yellow, "|").fmt(f)?;
					" ".fmt(f)?;
					command.fmt(f)?;
				}
			},
		};

		Ok(())
	}
}


impl std::fmt::Display for CommandBlockKind {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Synchronous => "{",
			Self::Asynchronous => "&{",
			Self::Capture => "${",
		}.fmt(f)
	}
}


impl<'a> Display<'a> for CommandBlock {
	type Context = Context<'a>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		self.kind.fmt(f)?;

		let nested = context.indent();

		fmt::sep_by(
			std::iter::once(&self.head).chain(self.tail.iter()),
			f,
			|cmd, f| {
				step(f, nested)?;
				cmd.fmt(f)
			},
			";",
		)?;

		step(f, context)?;

		"}".fmt(f)
	}
}


impl<'a> Display<'a> for Program {
	type Context = Context<'a>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		if context.indentation.is_some() {
			writeln!(
				f,
				"{} for {}",
				color::Fg(color::Yellow, "Program"),
				fmt::Show(self.source, context.interner)
			)?;
		}

		let root_frame = mem::FrameInfo {
			slots: self.root_slots,
			captures: Box::default(),
			self_slot: None,
		};

		root_frame.fmt(f, context.indentation)?;
		step(f, context)?;
		self.statements.fmt(f, context)
	}
}


fn step(f: &mut std::fmt::Formatter, ctx: Context) -> std::fmt::Result {
	if let Some(indent) = ctx.indentation {
		"\n".fmt(f)?;
		indent.fmt(f)
	} else {
		" ".fmt(f)
	}
}
