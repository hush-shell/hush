use std::{
	convert::TryInto,
	hash::{Hash, Hasher},
	ops::Deref,
};

use gc::{Gc, GcCell, GcCellRef, GcCellRefMut, Finalize, Trace};

use super::{EmptyCollection, IndexOutOfBounds, Value};


/// An array in the language.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Trace, Finalize)]
pub struct Array(Gc<GcCell<Vec<Value>>>);


impl Array {
	/// Crate a new empty array.
	pub fn new(vec: Vec<Value>) -> Self {
		Self(Gc::new(GcCell::new(vec)))
	}


	/// Shallow copy.
	pub fn copy(&self) -> Self {
		Self(self.0.clone())
	}


	/// Borrow the inner Vec.
	pub fn borrow(&self) -> GcCellRef<Vec<Value>> {
		self.0.deref().borrow()
	}


	/// Borrow the inner Vec mutably.
	pub fn borrow_mut(&self) -> GcCellRefMut<Vec<Value>> {
		self.0.deref().borrow_mut()
	}


	/// Push a value into the array.
	pub fn push(&mut self, value: Value) {
		self.0.borrow_mut().push(value)
	}


	/// Pop a value from the back of the array.
	pub fn pop(&mut self) -> Result<Value, EmptyCollection> {
		self.0
			.borrow_mut()
			.pop()
			.ok_or(EmptyCollection)
	}


	/// Get the value at a given index.
	pub fn index(&self, index: i64) -> Result<Value, IndexOutOfBounds> {
		let index: usize = index
			.try_into()
			.map_err(|_| IndexOutOfBounds)?;

		self
			.borrow()
			.get(index)
			.map(Value::copy)
			.ok_or(IndexOutOfBounds)
	}


	/// Check if the collections contains the given value
	pub fn contains(&self, value: &Value) -> bool {
		self
			.borrow()
			.contains(value)
	}


	/// Assign a value to the given index.
	pub fn set(&self, index: i64, value: Value) -> Result<(), IndexOutOfBounds> {
		let index: usize = index
			.try_into()
			.map_err(|_| IndexOutOfBounds)?;

		let mut array = self.borrow_mut();

		let val = array
			.get_mut(index)
			.ok_or(IndexOutOfBounds)?;

		*val = value;

		Ok(())
	}


	/// Get the array length.
	pub fn len(&self) -> i64 {
		self.borrow().len() as i64
	}


	/// Whether the array is empty.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Stable sort
	pub fn sort(&mut self) {
		self.borrow_mut().sort();
	}
}


// GcCell does not implement Eq because `borrow` might panic.
#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for Array {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.borrow().hash(state)
	}
}
