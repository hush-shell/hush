use super::{Error, Errors, AnalysisDisplayContext};
use crate::{
	fmt::{self, Display},
	symbol,
	term::color
};


impl<'a> Display<'a> for Error {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Lexer(error) => error.fmt(f, context),
			Self::Parser(error) => error.fmt(f, context),
		}
	}
}


/// We need this in order to be able to implement std::error::Error.
impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		Display::fmt(self, f, &symbol::Interner::new())
	}
}


impl<'a> Display<'a> for Errors {
	type Context = AnalysisDisplayContext<'a>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		for (ix, error) in self.0.into_iter().enumerate() {
			if let Some(max) = context.max_errors {
				if max <= ix {
					writeln!(
						f,
						"{} {}",
						color::Fg(color::Red, max),
						color::Fg(color::Red, "more supressed syntax errors"),
					)?;

					break;
				}
			}

			writeln!(
				f,
				"{}: {}",
				color::Fg(color::Red, "Error"),
				fmt::Show(error, context.interner)
			)?;
		}

		Ok(())
	}
}
