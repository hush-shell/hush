use std::fmt::{self, Display};


/// A human readable position in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos {
	pub line: u32,
	pub column: u32,
}


impl Pos {
	pub fn visit(&mut self, input: u8) {
		if input == b'\n' {
			self.line += 1;
			self.column = 0;
		} else {
			self.column += 1;
		}
	}
}


impl Default for Pos {
	fn default() -> Self {
		Self { line: 1, column: 0 }
	}
}


impl Display for Pos {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "line {}, column {}", self.line, self.column)
	}
}
