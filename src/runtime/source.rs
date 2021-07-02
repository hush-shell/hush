use std::cmp::Ordering;

use crate::{
	fmt::{self, Display},
	syntax,
	symbol::{self, Symbol},
};

use gc::{Finalize, Trace};


/// A human readable position in the source code.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Finalize)]
pub struct SourcePos {
	pub line: u32,
	pub column: u32,
	pub path: Symbol,
}


impl SourcePos {
	/// Shallow copy.
	pub fn copy(&self) -> Self {
		Self { .. *self }
	}


	/// Create a new SourcePos refering to the beginning of the file.
	pub fn file(path: Symbol) -> Self {
		Self { line: 0, column: 0, path }
	}
}


/// SourcePos has not garbage-collected fields.
unsafe impl Trace for SourcePos {
	gc::unsafe_empty_trace!();
}


impl PartialOrd for SourcePos {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}


impl Ord for SourcePos {
	fn cmp(&self, other: &Self) -> Ordering {
		(usize::from(self.path), self.line, self.column)
			.cmp(&(other.path.into(), other.line, other.column))
	}
}


impl From<syntax::SourcePos> for SourcePos {
	fn from(pos: syntax::SourcePos) -> Self {
		Self {
			line: pos.line,
			column: pos.column,
			path: pos.path,
		}
	}
}


impl<'a> From<&'a syntax::SourcePos> for SourcePos {
	fn from(pos: &'a syntax::SourcePos) -> Self {
		(*pos).into()
	}
}


impl<'a> Display<'a> for SourcePos {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		write!(
			f,
			"{} (line {}, column {})",
			fmt::Show(self.path, context),
			self.line,
			self.column
		)
	}
}
