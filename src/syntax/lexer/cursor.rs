use std::fmt::{self, Display};


/// A human readable position in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourcePos {
	pub line: u32,
	pub column: u32,
}


impl SourcePos {
	pub fn visit(&mut self, input: u8) {
		if input == b'\n' {
			self.line += 1;
			self.column = 0;
		} else {
			self.column += 1;
		}
	}
}


impl Default for SourcePos {
	fn default() -> Self {
		Self { line: 1, column: 0 }
	}
}


impl Display for SourcePos {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "line {}, column {}", self.line, self.column)
	}
}


/// A cursor for the source code.
#[derive(Debug, Clone)]
pub struct Cursor<'a> {
	input: &'a [u8],
	offset: usize,
	pos: SourcePos,
}


impl<'a> Cursor<'a> {
	pub fn pos(&self) -> SourcePos {
		self.pos
	}


	pub fn offset(&self) -> usize {
		self.offset
	}


	pub fn is_eof(&self) -> bool {
		self.offset == self.input.len()
	}


	pub fn peek(&self) -> Option<u8> {
		self.input.get(self.offset).copied()
	}


	pub fn slice(&self) -> &'a [u8] {
		&self.input
	}


	pub fn step(&mut self) {
		if self.is_eof() {
			return;
		}

		self.pos.visit(self.input[self.offset]);
		self.offset += 1;
	}
}


impl<'a> From<&'a [u8]> for Cursor<'a> {
	fn from(input: &'a [u8]) -> Self {
		Self {
			input,
			offset: 0,
			pos: SourcePos::default()
		}
	}
}
