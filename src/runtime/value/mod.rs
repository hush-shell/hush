#[macro_use]
mod ops;
mod array;
mod dict;
mod errors;
mod float;
mod fmt;
mod function;

use gc::{Gc, Finalize, Trace};

use super::{
	program,
	mem,
	panic::Panic,
	source::SourcePos,
};
pub use array::Array;
pub use dict::Dict;
pub use function::{Function, HushFun, RustFun};
pub use float::Float;
pub use errors::IndexOutOfBounds;


/// A value of dynamic type in the language.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Trace, Finalize)]
pub enum Value {
	Nil,
	Bool(bool),
	Int(i64),
	Float(Float),
	Byte(u8),
	/// Strings are immutable.
	String(Gc<Box<[u8]>>),
	Array(Array),
	Dict(Dict),
	Function(Gc<Function>),
	Error(), // TODO
}


impl Value {
	/// Make a shallow copy of the value.
	pub fn copy(&self) -> Self {
		match self {
			Self::Nil => Self::Nil,
			Self::Bool(b) => Self::Bool(*b),
			Self::Int(int) => Self::Int(*int),
			Self::Float(float) => Self::Float(float.clone()),
			Self::Byte(byte) => Self::Byte(*byte),
			Self::String(string) => Self::String(string.clone()),
			Self::Array(array) => Self::Array(array.copy()),
			Self::Dict(dict) => Self::Dict(dict.copy()),
			Self::Function(fun) => Self::Function(fun.clone()),
			Self::Error() => todo!(),
		}
	}
}


impl Default for Value {
	/// The default value is Nil.
	fn default() -> Self {
		Self::Nil
	}
}


macro_rules! from_variant {
	($variant: ident, $type: ident) => {
		impl From<$type> for Value {
			fn from(value: $type) -> Self {
				Self::$variant(value.into())
			}
		}
	}
}

from_variant!(Bool, bool);
from_variant!(Int, i64);
from_variant!(Float, f64);
from_variant!(Float, Float);
from_variant!(Byte, u8);
from_variant!(Array, Array);
from_variant!(Dict, Dict);


impl<'a> From<&'a [u8]> for Value {
	fn from(string: &'a [u8]) -> Self {
		Self::String(
			Gc::new(string.into())
		)
	}
}


impl From<Box<[u8]>> for Value {
	fn from(string: Box<[u8]>) -> Self {
		Self::String(
			Gc::new(string)
		)
	}
}


impl<'a> From<&'a str> for Value {
	fn from(string: &'a str) -> Self {
		string.as_bytes().into()
	}
}


impl From<Box<str>> for Value {
	fn from(string: Box<str>) -> Self {
		let boxed: Box<[u8]> = string.into();
		boxed.into()
	}
}


impl From<String> for Value {
	fn from(string: String) -> Self {
		string.into_boxed_str().into()
	}
}


impl From<Function> for Value {
	fn from(fun: Function) -> Self {
		Self::Function(Gc::new(fun))
	}
}


impl From<HushFun> for Value {
	fn from(fun: HushFun) -> Self {
		let fun: Function = fun.into();
		fun.into()
	}
}


impl From<RustFun> for Value {
	fn from(fun: RustFun) -> Self {
		let fun: Function = fun.into();
		fun.into()
	}
}
