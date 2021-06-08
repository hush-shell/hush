use super::{ast, Analysis};
use crate::{
	fmt::Display,
	symbol,
	term::color,
};


impl<'a> Display<'a> for Analysis {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		for error in self.errors.iter() {
			write!(f, "{}: ", color::Fg(color::Red, "Error"))?;
			error.fmt(f, context)?;
		}

		self.ast.fmt(f, ast::fmt::Context::from(context))
	}
}
