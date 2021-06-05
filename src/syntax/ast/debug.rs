use std::fmt::{self, Debug};

use super::{
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
use crate::symbol::{Symbol, SymbolExt};


impl Debug for Block {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if f.alternate() {
			write!(f, "#")?;
			f.debug_set().entries(self.0.iter()).finish()?;
		} else {
			for statement in self.0.iter() {
				write!(f, "{:?}; ", statement)?;
			}
		}

		Ok(())
	}
}


impl Debug for Literal {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Nil => write!(f, "nil"),
			Self::Bool(b) => write!(f, "{}", b),
			Self::Int(i) => write!(f, "{}", *i),
			Self::Float(n) => write!(f, "{}", *n),
			Self::Byte(c) => write!(f, "'{}'", *c as char),
			Self::String(s) => write!(f, "\"{}\"", String::from_utf8_lossy(s)),
			Self::Array(arr) => f.debug_list().entries(arr.iter()).finish(),
			Self::Dict(dict) => {
				write!(f, "@")?;
				f
					.debug_list()
					.entries(
						dict
							.iter()
							.map(
								|(key, expr)| {
									struct Pair<'a> { key: &'a Symbol, expr: &'a Expr }

									impl<'a> Debug for Pair<'a> {
										fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
											if f.alternate() {
												write!(f, "id#{}: {:#?}", self.key.to_usize(), self.expr)
											} else {
												write!(f, "id#{}: {:?}", self.key.to_usize(), self.expr)
											}
										}
									}

									Pair { key, expr }
								}
							)
					)
					.finish()
			},
			Self::Function { args, body } => {
				write!(f, "function (")?;

				for arg in args.iter() {
					write!(f, "id#{}, ", arg.to_usize())?;
				}

				if f.alternate() {
					write!(f, ") {:#?}", body)
				} else {
					write!(f, ") {:?} end", body)
				}
			}
			Self::Identifier(identifier) => write!(f, "\"id#{}\"", identifier.to_usize()),
		}
	}
}


impl Debug for UnaryOp {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(match self {
			Self::Minus => "-",
			Self::Not => "not",
		})
	}
}


impl Debug for BinaryOp {
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
			Self::And => "and",
			Self::Or => "or",
			Self::Concat => "++",
		})
	}
}


impl Debug for Expr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::IllFormed => write!(f, "***ill-formed***"),
			Self::Self_ { .. } => write!(f, "self"),
			Self::Identifier { identifier, .. } => write!(f, "id#{}", identifier.to_usize()),
			Self::Literal { literal, .. } => {
				if f.alternate() {
					write!(f, "{:#?}", literal)
				} else {
					write!(f, "{:?}", literal)
				}
			}
			Self::UnaryOp { op, operand, .. } => write!(f, "({:?} {:?})", op, operand),
			Self::BinaryOp { left, op, right, .. } => {
				write!(f, "({:?} {:?} {:?})", left, op, right)
			}
			Self::If { condition, then, otherwise, .. } => {
				if f.alternate() {
					write!(f, "if {:?} {:#?} else {:#?}", condition, then, otherwise)
				} else {
					write!(
						f,
						"if {:?} then {:?} else {:?} end",
						condition, then, otherwise
					)
				}
			}
			Self::Access { object, field, .. } => write!(f, "{:?}[{:?}]", object, field),
			Self::Call { function, params, .. } => {
				write!(f, "{:?}(", function)?;

				for param in params.iter() {
					write!(f, "{:?}, ", param)?;
				}

				write!(f, ")")
			}
			Self::CommandBlock { kind, commands, .. } => {
				match kind {
					CommandBlockKind::Synchronous => (),
					CommandBlockKind::Asynchronous => write!(f, "&")?,
					CommandBlockKind::Capture => write!(f, "$")?,
				}

				f.debug_set().entries(commands.iter()).finish()
			}
		}
	}
}


impl Debug for Statement {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::IllFormed => write!(f, "***ill-formed***"),
			Self::Let { identifier, init, .. } => {
				if f.alternate() {
					write!(f, "let id#{} = {:#?}", identifier.to_usize(), init)
				} else {
					write!(f, "let id#{} = {:?}", identifier.to_usize(), init)
				}
			}
			Self::Assign { left, right, .. } => {
				if f.alternate() {
					write!(f, "{:?} = {:#?}", left, right)
				} else {
					write!(f, "{:?} = {:?}", left, right)
				}
			}
			Self::Return { expr, .. } => {
				if f.alternate() {
					write!(f, "return {:#?}", expr)
				} else {
					write!(f, "return {:?}", expr)
				}
			}
			Self::Break { .. } => write!(f, "break"),
			Self::While { condition, block, .. } => {
				if f.alternate() {
					write!(f, "while {:?} do {:#?}", condition, block)
				} else {
					write!(f, "while {:?} do {:?} end", condition, block)
				}
			}
			Self::For { identifier, expr, block, .. } => {
				if f.alternate() {
					write!(
						f,
						"for id#{} in {:?} do {:#?}",
						identifier.to_usize(),
						expr,
						block
					)
				} else {
					write!(
						f,
						"for id#{} in {:?} do {:?} end",
						identifier.to_usize(),
						expr,
						block
					)
				}
			}
			Self::Expr(expr) => {
				if f.alternate() {
					write!(f, "{:#?}", expr)
				} else {
					write!(f, "{:?}", expr)
				}
			}
		}
	}
}


impl Debug for ArgUnit {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Literal(lit) => write!(f, "{}", String::from_utf8_lossy(lit)),
			Self::Dollar(identifier) => write!(f, "${{id#{}}}", identifier.to_usize()),
		}
	}
}


impl Debug for ArgPart {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Unit(unit) => write!(f, "{:?}", unit),
			Self::Home => write!(f, "~/"),
			Self::Range(start, end) => write!(f, "{{{}..{}}}", start, end),
			Self::Collection(items) => f.debug_set().entries(items.iter()).finish(),
			Self::Star => write!(f, "*"),
			Self::Question => write!(f, "?"),
			Self::CharClass(chars) => write!(f, "[{}]", String::from_utf8_lossy(chars)),
		}
	}
}


impl Debug for Argument {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "\"")?;

		for part in self.parts.iter() {
			write!(f, "{:?}", part)?;
		}

		write!(f, "\"")
	}
}


impl Debug for RedirectionTarget {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Fd(fd) => write!(f, ">{}", fd),
			Self::Overwrite(arg) => write!(f, ">{:?}", arg),
			Self::Append(arg) => write!(f, ">>{:?}", arg),
		}
	}
}


impl Debug for Redirection {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::IllFormed => write!(f, "***ill-formed***"),
			Self::Output { source, target } => write!(f, "{}{:?}", source, target),
			Self::Input { literal: false, source } => write!(f, "<{:?}", source),
			Self::Input { literal: true, source } => write!(f, "<<{:?}", source),
		}
	}
}


impl Debug for BasicCommand {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.command)?;

		for arg in self.arguments.iter() {
			write!(f, " {:?}", arg)?;
		}

		for redirection in self.redirections.iter() {
			write!(f, " {:?}", redirection)?;
		}

		if self.abort_on_error {
			write!(f, " ?")?;
		}

		Ok(())
	}
}


impl Debug for Command {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut commands = self.0.iter();

		let command = commands.next().expect("empty command");

		write!(f, "{:?}", command)?;

		for command in commands {
			write!(f, " | {:?}", command)?;
		}

		Ok(())
	}
}


impl Debug for Ast {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if f.alternate() {
			writeln!(f, "AST for {}", self.path.display())?;
			writeln!(f, "{:#?}", self.statements)
		} else {
			writeln!(f, "{:?}", self.statements)
		}
	}
}
