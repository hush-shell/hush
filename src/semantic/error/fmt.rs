use std::fmt::Display as _;

use super::{Errors, Error, ErrorKind};
use crate::{
	fmt::{self, Display},
	symbol::{self},
	term::color
};


/// Context for displaying errors.
#[derive(Debug, Copy, Clone)]
pub struct ErrorsDisplayContext<'a> {
	/// Max number of displayed errors.
	pub max_errors: Option<usize>,
	/// Symbol interner.
	pub interner: &'a symbol::Interner,
}


impl<'a> Display<'a> for ErrorKind {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::UndeclaredVariable(symbol) => {
				"undeclared variable '".fmt(f)?;
				symbol.fmt(f, context)?;
				"'".fmt(f)
			}

			Self::DuplicateVariable(symbol) => {
				"duplicate variable '".fmt(f)?;
				symbol.fmt(f, context)?;
				"'".fmt(f)
			}

			Self::DuplicateKey(symbol) => {
				"duplicate key '".fmt(f)?;
				symbol.fmt(f, context)?;
				"'".fmt(f)
			}

			Self::ReturnOutsideFunction => write!(f, "return statement outside function"),

			Self::SelfOutsideFunction => write!(f, "self variable outside function"),

			Self::BreakOutsideLoop => write!(f, "break statement outside loop"),

			Self::InvalidAssignment => write!(f, "invalid assignment"),

			Self::AsyncBuiltin => write!(f, "use of built-in command in async context"),
		}
	}
}


impl<'a> Display<'a> for Error {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		write!(f, "{}: {} - ", color::Fg(color::Red, "Error"), fmt::Show(self.pos, context))?;
		self.kind.fmt(f, context)
	}
}


/// We need this in order to be able to implement std::error::Error.
impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		Display::fmt(self, f, &symbol::Interner::new())
	}
}


impl<'a> Display<'a> for Errors {
	type Context = ErrorsDisplayContext<'a>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		for (ix, error) in self.0.iter().enumerate() {
			if let Some(max) = context.max_errors {
				if max <= ix {
					writeln!(
						f,
						"{} {}",
						color::Fg(color::Red, max),
						color::Fg(color::Red, "more supressed semantic errors"),
					)?;

					break;
				}
			}

			writeln!(f, "{}", fmt::Show(error, context.interner))?;
		}

		Ok(())
	}
}


/// We need this in order to be able to implement std::error::Error.
impl std::fmt::Display for Errors {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		for error in self.0.iter() {
			Display::fmt(error, f, &symbol::Interner::new())?;
		}

		Ok(())
	}
}
