use std::{
	fmt::{self, Display},
	fs::File,
	path::Path,
};


/// Hush source code.
#[derive(Debug)]
pub struct Source {
	/// The origin path, may be something fictional like `<stdin>`.
	pub path: Box<Path>,
	/// The source code.
	pub contents: Box<[u8]>,
}


impl Source {
	/// Load the source code from a file path.
	pub fn from_path<P>(path: P) -> std::io::Result<Self>
	where
		P: Into<Box<Path>>,
	{
		let path = path.into();
		let file = File::open(&path)?;
		Self::from_reader(path, file)
	}


	/// Load the source code from a std::io::Read.
	/// The path argument may be anything, including fictional paths like `<stdin>`.
	pub fn from_reader<P, R>(path: P, mut reader: R) -> std::io::Result<Self>
	where
		P: Into<Box<Path>>,
		R: std::io::Read,
	{
		let path = path.into();
		let mut contents = Vec::with_capacity(512); // Expect a few characters.
		reader.read_to_end(&mut contents)?;

		Ok(Self { path, contents: contents.into() })
	}
}


/// A human readable position in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourcePos {
	pub line: u32,
	pub column: u32,
}


impl Display for SourcePos {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "line {}, column {}", self.line, self.column)
	}
}
