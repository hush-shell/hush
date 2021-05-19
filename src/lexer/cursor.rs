use crate::source::Pos as SourcePos;


/// A cursor for the source code.
#[derive(Debug, Clone)]
pub struct Cursor<'a> {
	input: &'a [u8],
	offset: usize,
	pos: SourcePos,
}


impl<'a> Cursor<'a> {
	pub fn new(input: &'a [u8]) -> Self {
		Self { input, offset: 0, pos: SourcePos::default() }
	}


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
		if self.input.len() == self.offset {
			return;
		}

		self.pos.visit(self.input[self.offset]);
		self.offset += 1;
	}
}
