use super::{ast, Analysis, Errors};
use crate::{
	fmt::{self, Display},
	symbol,
	term::color,
};


/// Context for displaying the syntax analysis.
#[derive(Debug, Copy, Clone)]
pub struct AnalysisDisplayContext<'a> {
	/// Max number of displayed errors.
	pub max_errors: Option<usize>,
	/// Symbol interner.
	pub interner: &'a symbol::Interner,
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


impl<'a> Display<'a> for Analysis {
	type Context = AnalysisDisplayContext<'a>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		self.errors.fmt(f, context)?;
		self.ast.fmt(f, ast::fmt::Context::from(context.interner))
	}
}
