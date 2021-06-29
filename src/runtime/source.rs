use std::{
	fmt::{self, Display},
	path::Path,
};

use crate::syntax;

use gc::{Finalize, Trace};


/// A human readable position in the source code.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Trace, Finalize)]
pub struct SourcePos {
	pub path: &'static Path,
	pub line: u32,
	pub column: u32,
}


impl SourcePos {
	/// Create a new SourcePos from the given AST SourcePos and source file path.
	pub fn new(pos: syntax::SourcePos, path: &'static Path) -> Self {
		Self {
			line: pos.line,
			column: pos.column,
			path,
		}
	}


	/// Shallow copy.
	pub fn copy(&self) -> Self {
		Self { .. *self }
	}


	/// Create a new SourcePos refering to the beginning of the file.
	pub fn file(path: &'static Path) -> Self {
		Self {
			line: 0,
			column: 0,
			path,
		}
	}
}


impl Display for SourcePos {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} (line {}, column {})", self.path.display(), self.line, self.column)
	}
}
