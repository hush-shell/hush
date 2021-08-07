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

use std::{cmp::Ordering, ffi::{OsString}};

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


/// A value of dynamic type in the language.
#[derive(Debug, Hash)]
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
}


impl PartialOrd for Value {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}


impl Ord for Value {
	fn cmp(&self, other: &Self) -> Ordering {
		let order = |value| match value {
			&Self::Nil => 0,
			&Self::Bool(_) => 1,
			&Self::Byte(_) => 2,
			// Int and float are comparable.
			&Self::Int(_) => 3,
			&Self::Float(_) => 3,
			&Self::String(_) => 4,
			&Self::Array(_) => 5,
			&Self::Dict(_) => 6,
			&Self::Function(_) => 7,
			&Self::Error(_) => 8,
		};

		match (self, other) {
			(Self::Nil, Self::Nil) => Ordering::Equal,

			(Self::Bool(bool1), Self::Bool(bool2)) => bool1.cmp(&bool2),

			(Self::Byte(byte1), Self::Byte(byte2)) => byte1.cmp(&byte2),

			(Self::Int(int1), Self::Int(int2)) => int1.cmp(&int2),

			(Self::Float(ref float1), Self::Float(ref float2)) => float1.cmp(float2),

			// Float and int are comparable:
			(Self::Int(int), Self::Float(ref float)) => Float::from(int).cmp(float),
			(Self::Float(ref float), Self::Int(int)) => float.cmp(&int.into()),

			(Self::String(ref str1), Self::String(ref str2)) => str1.cmp(&str2),

			(Self::Array(array1), Self::Array(array2)) => array1.cmp(&array2),

			(Self::Dict(dict1), Self::Dict(dict2)) => dict1.cmp(&dict2),

			(Self::Function(function1), Self::Function(function2)) => function1.cmp(&function2),

			(Self::Error(error1), Self::Error(error2)) => error1.cmp(&error2),

			_ => order(self).cmp(&order(other)),
		}
	}
}


impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Nil, Self::Nil) => true,

			(Self::Bool(bool1), Self::Bool(bool2)) => bool1.eq(&bool2),

			(Self::Byte(byte1), Self::Byte(byte2)) => byte1.eq(&byte2),

			(Self::Int(int1), Self::Int(int2)) => int1.eq(&int2),

			(Self::Float(ref float1), Self::Float(ref float2)) => float1.eq(float2),

			// Float and int are comparable:
			(Self::Int(int), Self::Float(ref float)) => Float::from(int).eq(float),
			(Self::Float(ref float), Self::Int(int)) => float.eq(&int.into()),

			(Self::String(ref str1), Self::String(ref str2)) => str1.eq(&str2),

			(Self::Array(array1), Self::Array(array2)) => array1.eq(&array2),

			(Self::Dict(dict1), Self::Dict(dict2)) => dict1.eq(&dict2),

			(Self::Function(function1), Self::Function(function2)) => function1.eq(&function2),

			(Self::Error(error1), Self::Error(error2)) => error1.eq(&error2),

			_ => false,
		}
	}
}


impl Eq for Value { }


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
