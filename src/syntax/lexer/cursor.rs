use super::{Source, SourcePos};


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

		if self.input[self.offset] == b'\n' {
			self.pos.line += 1;
			self.pos.column = 0;
		} else {
			self.pos.column += 1;
		}

		self.offset += 1;
	}


	/// Save a checkpoint in the current position.
	pub fn checkpoint(&self) -> Checkpoint {
		Checkpoint {
			offset: self.offset,
			pos: self.pos,
		}
	}


	/// Rollback to the given checkpoint.
	pub fn rollback(&mut self, checkpoint: Checkpoint) {
		self.offset = checkpoint.offset;
		self.pos = checkpoint.pos;
	}
}


impl<'a> From<&'a Source> for Cursor<'a> {
	fn from(source: &'a Source) -> Self {
		Self {
			input: &source.contents,
			offset: 0,
			pos: SourcePos { line: 1, column: 0, path: source.path }
		}
	}
}


/// A cursor checkpoint.
/// This can be used to save and restore a position.
#[derive(Debug, Copy, Clone)]
pub struct Checkpoint {
	offset: usize,
	pos: SourcePos,
}
