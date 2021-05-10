use std::fmt::{self, Display};


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourcePos {
	pub line: u32,
	pub column: u32,
}


impl SourcePos {
	pub fn forward_line(&mut self) {
		self.line += 1;
		self.column = 0;
	}

	pub fn forward_column(&mut self) {
		self.column += 1;
	}

	pub fn visit(&mut self, input: &[u8]) {
		let (lines, last_line_len) = input
			.split(|&c| c == b'\n')
			.enumerate()
			.last()
			.map(|(lines, last_line)| (lines as u32, last_line.len() as u32))
			.expect("split should always yield at least once");

		self.line += lines;
		self.column = if lines == 0 {
			self.column + last_line_len
		} else {
			last_line_len
		}
	}
}


impl Display for SourcePos {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "line {}, column {}", self.line, self.column)
	}
}


#[derive(Debug, Clone)]
pub struct Cursor<'a> {
	input: &'a [u8],
	pos: SourcePos,
}


impl<'a> Cursor<'a> {
	pub fn new(input: &'a [u8]) -> Self {
		Self {
			input,
			pos: SourcePos::default(),
		}
	}


	pub fn pos(&self) -> SourcePos {
		self.pos
	}


	pub fn eat(&mut self, pattern: &[u8]) -> bool {
		if let Some(remainder) = self.input.strip_prefix(pattern) {
			self.input = remainder;
			self.pos.visit(pattern);
			true
		} else {
			false
		}
	}


	pub fn peek(&self) -> Option<u8> {
		self.input.first().copied()
	}


	pub fn take<P>(&mut self, mut predicate: P) -> Option<&'a [u8]>
	where
		P: FnMut(u8) -> bool,
	{
		if self.input.is_empty() {
			return None;
		}

		// Find the index of the first character that does not match.
		let result = self.input.iter().position(|&c| {
			if c == b'\n' {
				self.pos.forward_line();
			} else {
				self.pos.forward_column();
			}

			!predicate(c)
		});

		let (prefix, remainder) = self.input.split_at(
			// If such character was not found, it means the entire input must be consumed.
			result.unwrap_or(self.input.len()),
		);

		self.input = remainder;

		Some(prefix)
	}


	pub fn skip(&mut self) -> Option<u8> {
		let (&first, remainder) = self.input.split_first()?;

		if first == b'\n' {
			self.pos.forward_line();
		} else {
			self.pos.forward_column();
		}

		self.input = remainder;

		Some(first)
	}


	pub fn skip_line(&mut self) -> Option<()> {
		self.take(|c| c != b'\n')?;
		Some(())
	}
}
