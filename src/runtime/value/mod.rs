mod fmt;
mod float;
mod function;

use std::{
	cmp::Ordering,
	collections::{HashMap, BTreeMap},
	convert::TryInto,
	hash::{Hash, Hasher},
	ops::Deref,
};

use gc::{Gc, GcCell, GcCellRef, GcCellRefMut, Finalize, Trace};

use super::{
	program,
	mem,
	panic::Panic,
	source::SourcePos,
};
pub use function::{Function, HushFun, RustFun};
pub use float::Float;


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Trace, Finalize)]
pub enum Value {
	Nil,
	Bool(bool),
	Int(i64),
	Float(Float),
	Byte(u8),
	String(Gc<Box<[u8]>>),
	Array(Array),
	Dict(Dict),
	Function(Gc<Function>),
	Error(), // TODO
}


impl Value {
	/// Shallow copy.
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
	fn default() -> Self {
		Self::Nil
	}
}


impl From<bool> for Value {
	fn from(b: bool) -> Self {
		Self::Bool(b)
	}
}


impl From<i64> for Value {
	fn from(int: i64) -> Self {
		Self::Int(int)
	}
}


impl From<f64> for Value {
	fn from(float: f64) -> Self {
		Self::Float(float.into())
	}
}


impl From<Float> for Value {
	fn from(float: Float) -> Self {
		Self::Float(float)
	}
}


impl From<u8> for Value {
	fn from(byte: u8) -> Self {
		Self::Byte(byte)
	}
}


impl<'a> From<&'a str> for Value {
	fn from(string: &'a str) -> Self {
		string.as_bytes().into()
	}
}


impl<'a> From<&'a [u8]> for Value {
	fn from(string: &'a [u8]) -> Self {
		Self::String(
			Gc::new(string.into())
		)
	}
}


impl From<Array> for Value {
	fn from(array: Array) -> Self {
		Self::Array(array)
	}
}


impl From<Dict> for Value {
	fn from(dict: Dict) -> Self {
		Self::Dict(dict)
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


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Trace, Finalize)]
pub struct Array(Gc<GcCell<Vec<Value>>>);


impl Array {
	pub fn new(vec: Vec<Value>) -> Self {
		Self(Gc::new(GcCell::new(vec)))
	}


	pub fn copy(&self) -> Self {
		Self(self.0.clone())
	}


	pub fn borrow(&self) -> GcCellRef<Vec<Value>> {
		self.0.deref().borrow()
	}


	pub fn borrow_mut(&self) -> GcCellRefMut<Vec<Value>> {
		self.0.deref().borrow_mut()
	}


	pub fn push(&mut self, value: Value) {
		self.0.borrow_mut().push(value)
	}


	pub fn index(&self, index: i64) -> Option<Value> {
		let index: usize = index.try_into().ok()?;

		self
			.borrow()
			.get(index)
			.map(Value::copy)
	}


	pub fn set(&self, index: i64, value: Value) {
		let index: usize = index.try_into().ok().expect("index overflow");

		self.borrow_mut()[index] = value;
	}


	pub fn len(&self) -> i64 {
		self.borrow().len() as i64
	}
}


impl Hash for Array {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.borrow().hash(state)
	}
}


#[derive(Debug, PartialEq, Eq)]
#[derive(Trace, Finalize)]
pub struct Dict(Gc<GcCell<HashMap<Value, Value>>>);


impl Dict {
	pub fn new(dict: HashMap<Value, Value>) -> Self {
		Self(Gc::new(GcCell::new(dict)))
	}


	pub fn copy(&self) -> Self {
		Self(self.0.clone())
	}


	pub fn borrow(&self) -> GcCellRef<HashMap<Value, Value>> {
		self.0.deref().borrow()
	}


	pub fn borrow_mut(&self) -> GcCellRefMut<HashMap<Value, Value>> {
		self.0.deref().borrow_mut()
	}


	pub fn insert(&self, key: Value, value: Value) {
		self.borrow_mut().insert(key, value);
	}


	pub fn get(&self, key: &Value) -> Option<Value> {
		self
			.borrow()
			.get(key)
			.map(Value::copy)
	}
}


impl PartialOrd for Dict {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}


/// This is very expensive, but there is no better way to correctly compare.
impl Ord for Dict {
	fn cmp(&self, other: &Self) -> Ordering {
		let _self = self.borrow();
		let _self: BTreeMap<&Value, &Value> = _self.iter().collect();

		let _other = other.borrow();
		let _other: BTreeMap<&Value, &Value> = _other.iter().collect();

		_self.cmp(&_other)
	}
}


impl Hash for Dict {
	fn hash<H: Hasher>(&self, state: &mut H) {
		let _self = self.borrow();
		let _self: BTreeMap<&Value, &Value> = _self.iter().collect();

		_self.hash(state)
	}
}
