use super::{ast, Analysis};
use crate::{
	fmt::Display,
	symbol,
};


/// Context for displaying the syntax analysis.
#[derive(Debug, Copy, Clone)]
pub struct AnalysisDisplayContext<'a> {
	/// Max number of displayed errors.
	pub max_errors: Option<usize>,
	/// Symbol interner.
	pub interner: &'a symbol::Interner,
}


impl<'a> Display<'a> for Analysis {
	type Context = AnalysisDisplayContext<'a>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		self.errors.fmt(f, context)?;
		self.ast.fmt(f, ast::fmt::Context::from(context.interner))
	}
}
