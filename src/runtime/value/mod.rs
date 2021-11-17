#[macro_use]
mod ops;
mod array;
mod dict;
mod error;
mod errors;
mod float;
mod fmt;
mod function;
mod string;

use std::{ffi::OsString, fmt::Display};

use gc::{Finalize, Trace};

use super::{
	program,
	mem,
	Panic,
	Runtime,
	SourcePos,
};
pub use array::Array;
pub use dict::{keys, Dict};
pub use error::Error;
pub use function::{CallContext, Function, HushFun, RustFun, NativeFun};
pub use float::Float;
pub use errors::{EmptyCollection, IndexOutOfBounds};
pub use string::Str;


/// The possible types in the language.
/// This enum is used mostly for utility, as the Value enum also encodes all types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
	Nil,
	Bool,
	Byte,
	Int,
	Float,
	String,
	Array,
	Dict,
	Function,
	Error,
}


impl Type {
	/// Parse from canonical name as ASCII string.
	pub fn parse<T>(input: T) -> Option<Self>
	where
		T: AsRef<[u8]>,
	{
		match input.as_ref() {
			b"nil" => Some(Self::Nil),
			b"bool" => Some(Self::Bool),
			b"int" => Some(Self::Int),
			b"float" => Some(Self::Float),
			b"char" => Some(Self::Byte),
			b"string" => Some(Self::String),
			b"array" => Some(Self::Array),
			b"dict" => Some(Self::Dict),
			b"function" => Some(Self::Function),
			b"error" => Some(Self::Error),
			_ => None,
		}
	}


	/// Convert to canonical name.
	pub fn display(&self) -> &'static str {
		match self {
			Self::Nil => "nil",
			Self::Bool => "bool",
			Self::Byte => "char",
			Self::Int => "int",
			Self::Float => "float",
			Self::String => "string",
			Self::Array => "array",
			Self::Dict => "dict",
			Self::Function => "function",
			Self::Error => "error",
		}
	}
}


impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_str(self.display())
	}
}


impl<'a> From<&'a Value> for Type {
	fn from(value: &'a Value) -> Self {
		value.get_type()
	}
}


/// A value of dynamic type in the language.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Trace, Finalize)]
pub enum Value {
	Nil,
	Bool(bool),
	Byte(u8),
	Int(i64),
	Float(Float),
	/// Strings are immutable.
	String(Str),
	Array(Array),
	Dict(Dict),
	Function(Function),
	Error(Error),
}


impl Value {
	/// Make a shallow copy of the value.
	pub fn copy(&self) -> Self {
		match self {
			Self::Nil => Self::Nil,
			Self::Bool(b) => Self::Bool(*b),
			Self::Int(int) => Self::Int(*int),
			Self::Float(float) => Self::Float(float.copy()),
			Self::Byte(byte) => Self::Byte(*byte),
			Self::String(string) => Self::String(string.copy()),
			Self::Array(array) => Self::Array(array.copy()),
			Self::Dict(dict) => Self::Dict(dict.copy()),
			Self::Function(fun) => Self::Function(fun.copy()),
			Self::Error(error) => Self::Error(error.copy())
		}
	}


	/// Get the type tag of the value.
	pub fn get_type(&self) -> Type {
		match self {
			Self::Nil => Type::Nil,
			Self::Bool(_) => Type::Bool,
			Self::Int(_) => Type::Int,
			Self::Float(_) => Type::Float,
			Self::Byte(_) => Type::Byte,
			Self::String(_) => Type::String,
			Self::Array(_) => Type::Array,
			Self::Dict(_) => Type::Dict,
			Self::Function(_) => Type::Function,
			Self::Error(_) => Type::Error,
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
from_variant!(String, Str);
from_variant!(Array, Array);
from_variant!(Dict, Dict);
from_variant!(Function, Function);
from_variant!(Error, Error);


impl From<()> for Value {
	fn from(_: ()) -> Self {
		Self::Nil
	}
}


impl<'a> From<&'a [u8]> for Value {
	fn from(string: &'a [u8]) -> Self {
		let string: Str = string.into();
		string.into()
	}
}


impl From<Box<[u8]>> for Value {
	fn from(string: Box<[u8]>) -> Self {
		let string: Str = string.into();
		string.into()
	}
}


impl From<OsString> for Value {
	fn from(string: OsString) -> Self {
		let string: Str = string.into();
		string.into()
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


impl From<Vec<Value>> for Value {
	fn from(array: Vec<Value>) -> Self {
		Self::Array(Array::new(array))
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

impl<T: NativeFun> From<T> for Value {
	fn from(fun: T) -> Self {
		let fun: Function = fun.into();
		fun.into()
	}
}


impl<T> From<Option<T>> for Value
where
	T: Into<Value>,
{
	fn from(option: Option<T>) -> Self {
		option
			.map(Into::into)
			.unwrap_or(Value::Nil)
	}
}


impl<T, E> From<Result<T, E>> for Value
where
	T: Into<Value>,
	E: Into<Error>,
{
	fn from(result: Result<T, E>) -> Self {
		match result {
			Ok(value) => value.into(),
			Err(error) => Value::Error(error.into()),
		}
	}
}
