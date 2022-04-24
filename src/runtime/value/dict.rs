use std::{
	cmp::Ordering,
	collections::{HashMap, BTreeMap},
	hash::{Hash, Hasher},
	ops::Deref,
};

use gc::{Gc, GcCell, GcCellRef, GcCellRefMut, Finalize, Trace};

use super::{IndexOutOfBounds, Value};


/// Common dict keys
pub mod keys {
	use super::Value;

	thread_local! {
		/// FINISHED string key.
		pub static FINISHED: Value = "finished".into();
		/// FINISHED string key.
		pub static KEY: Value = "key".into();
		/// VALUE string key.
		pub static VALUE: Value = "value".into();
	}
}


/// A dict in the language.
#[derive(Debug, Default, PartialEq, Eq)]
#[derive(Trace, Finalize)]
pub struct Dict(Gc<GcCell<HashMap<Value, Value>>>);


impl Dict {
	/// Crate a new empty dict.
	pub fn new(dict: HashMap<Value, Value>) -> Self {
		Self(Gc::new(GcCell::new(dict)))
	}


	/// Shallow copy.
	pub fn copy(&self) -> Self {
		Self(self.0.clone())
	}


	/// Borrow the hashmap.
	pub fn borrow(&self) -> GcCellRef<HashMap<Value, Value>> {
		self.0.deref().borrow()
	}


	/// Borrow the hashmap mutably.
	pub fn borrow_mut(&self) -> GcCellRefMut<HashMap<Value, Value>> {
		self.0.deref().borrow_mut()
	}


	/// Insert a value in the dict.
	pub fn insert(&self, key: Value, value: Value) {
		self.borrow_mut().insert(key, value);
	}


	/// Get the value for the given key.
	pub fn get(&self, key: &Value) -> Result<Value, IndexOutOfBounds> {
		self
			.borrow()
			.get(key)
			.map(Value::copy)
			.ok_or(IndexOutOfBounds)
	}


	/// Check if the collections contains the given key
	pub fn contains(&self, key: &Value) -> bool {
		self
			.borrow()
			.contains_key(key)
	}


	/// Get the dict length.
	pub fn len(&self) -> i64 {
		self.borrow().len() as i64
	}


	/// Whether the dict is empty.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}


/// We need PartialOrd in order to be able to store dicts as keys in other dicts.
impl PartialOrd for Dict {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}


/// We need Ord in order to be able to store dicts as keys in other dicts.
impl Ord for Dict {
	fn cmp(&self, other: &Self) -> Ordering {
		// This is very expensive, but there is no better way to correctly compare.
		let _self = self.borrow();
		let _self: BTreeMap<&Value, &Value> = _self.iter().collect();

		let _other = other.borrow();
		let _other: BTreeMap<&Value, &Value> = _other.iter().collect();

		_self.cmp(&_other)
	}
}


/// We need Hash in order to be able to store dicts as keys in other dicts.
// GcCell does not implement Eq because `borrow` might panic.
#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Dict {
	fn hash<H: Hasher>(&self, state: &mut H) {
		// This is very expensive, but there is no better way to correctly compare.
		let _self = self.borrow();
		let _self: BTreeMap<&Value, &Value> = _self.iter().collect();

		_self.hash(state)
	}
}
