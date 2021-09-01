use std::{
	ffi::OsStr,
	fmt::{self, Display},
	fs::File,
	os::unix::ffi::OsStrExt,
};

use crate::symbol::{self, Symbol};


/// Hush source code.
#[derive(Debug)]
pub struct Source {
	/// The origin path, may be something fictional like `<stdin>`.
	pub path: Symbol,
	/// The source code.
	pub contents: Box<[u8]>,
}


impl Source {
	/// Load the source code from a file path.
	pub fn from_path(symbol: Symbol, interner: &mut symbol::Interner) -> std::io::Result<Self> {
		let path = OsStr::from_bytes(
			interner
				.resolve(symbol)
				.expect("failed to resolve path symbol")
		);
		let file = File::open(path)?;
		Self::from_reader(symbol, file)
	}


	/// Load the source code from a std::io::Read.
	/// The path argument may be anything, including fictional paths like `<stdin>`.
	pub fn from_reader<R>(path: Symbol, mut reader: R) -> std::io::Result<Self>
	where
		R: std::io::Read,
	{
		let mut contents = Vec::with_capacity(512); // Expect a few characters.
		reader.read_to_end(&mut contents)?;

		Ok(Self { path, contents: contents.into() })
	}
}


/// A human readable position in the source code.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourcePos {
	pub line: u32,
	pub column: u32,
	pub path: Symbol,
}


impl Display for SourcePos {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "line {}, column {}", self.line, self.column)
	}
}
