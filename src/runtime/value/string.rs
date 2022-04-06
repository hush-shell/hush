use std::{
    convert::TryInto,
    ffi::{OsString, OsStr},
    ops::Deref,
    os::unix::ffi::{OsStringExt, OsStrExt},
    path::PathBuf,
};

use gc::{Gc, Finalize, Trace};

use super::{IndexOutOfBounds, Value};


/// Strings in Hush are immutable.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Trace, Finalize)]
pub struct Str(Gc<Box<[u8]>>);


impl Str {
	/// Shallow copy.
	pub fn copy(&self) -> Self {
		Self(self.0.clone())
	}


	/// Get the underlying slice.
	pub fn as_bytes(&self) -> &[u8] {
		self.as_ref()
	}


	/// Get the value at a given index.
	pub fn index(&self, index: i64) -> Result<Value, IndexOutOfBounds> {
		let index: usize = index
			.try_into()
			.map_err(|_| IndexOutOfBounds)?;

		self.0
			.get(index)
			.copied()
			.map(Value::Byte)
			.ok_or(IndexOutOfBounds)
	}


	/// Check if the collections contains the given value
	pub fn contains(&self, byte: u8) -> bool {
		self.0.contains(&byte)
	}


	/// Get the string length.
	pub fn len(&self) -> usize {
		self.0.len()
	}


	/// Whether the string is empty.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}


impl AsRef<[u8]> for Str {
	fn as_ref(&self) -> &[u8] {
		self.0.deref().deref()
	}
}


impl AsRef<OsStr> for Str {
	fn as_ref(&self) -> &OsStr {
		OsStr::from_bytes(self.as_ref())
	}
}


impl<'a> From<&'a [u8]> for Str {
	fn from(string: &'a [u8]) -> Self {
		Self(
			Gc::new(string.into())
		)
	}
}


impl From<Box<[u8]>> for Str {
	fn from(string: Box<[u8]>) -> Self {
		Self(
			Gc::new(string)
		)
	}
}


impl From<Vec<u8>> for Str {
	fn from(vec: Vec<u8>) -> Self {
		Self::from(vec.into_boxed_slice())
	}
}


impl<'a> From<&'a str> for Str {
	fn from(string: &'a str) -> Self {
		string.as_bytes().into()
	}
}


impl From<Box<str>> for Str {
	fn from(string: Box<str>) -> Self {
		let boxed: Box<[u8]> = string.into();
		boxed.into()
	}
}


impl From<String> for Str {
	fn from(string: String) -> Self {
		string.into_boxed_str().into()
	}
}


impl From<OsString> for Str {
	fn from(string: OsString) -> Self {
		string.into_vec().into_boxed_slice().into()
	}
}

impl From<PathBuf> for Str {
	fn from(path: PathBuf) -> Self {
		path.into_os_string().into()
	}
}
