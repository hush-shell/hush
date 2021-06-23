use std::{
	fmt::{self, Debug},
	ops::{Add, AddAssign, Sub, SubAssign, Deref},
};

use super::{program, Value};

use gc::{Gc, GcCell, Finalize, Trace};


/// The index of a memory slot in the activation record.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[derive(Trace, Finalize)]
pub struct SlotIx(pub u32);


impl Add for SlotIx {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self(self.0 + rhs.0)
	}
}


impl AddAssign for SlotIx {
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0
	}
}


impl Sub for SlotIx {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}


impl SubAssign for SlotIx {
	fn sub_assign(&mut self, rhs: Self) {
		self.0 -= rhs.0
	}
}



impl From<program::mem::SlotIx> for SlotIx {
	fn from(slot_ix: program::mem::SlotIx) -> Self {
		Self(slot_ix.0)
	}
}


impl From<&program::mem::SlotIx> for SlotIx {
	fn from(slot_ix: &program::mem::SlotIx) -> Self {
		Self(slot_ix.0)
	}
}


#[derive(Debug)]
enum Slot {
	Regular(Value),
	Closed(Gc<GcCell<Value>>),
}


impl Slot {
	fn fetch(&self) -> Value {
		match self {
			Slot::Regular(value) => value.copy(),
			Slot::Closed(value) => value.deref().borrow().copy(),
		}
	}


	fn store(&mut self, value: Value) {
		match self {
			Self::Closed(val) => *val.borrow_mut() = value,
			Self::Regular(val) => *val = value,
		}
	}


	fn capture(&mut self) -> Gc<GcCell<Value>> {
		match self {
			// Value already closed.
			Slot::Closed(value) => value.clone(),
			// Promote regular value to closed value.
			Slot::Regular(value) => {
				let value = Gc::new(GcCell::new(std::mem::take(value)));
				*self = Slot::Closed(value.clone());
				value
			}
		}
	}


	fn place(&mut self, value: Gc<GcCell<Value>>) {
		*self = Slot::Closed(value);
	}
}


impl Default for Slot {
	fn default() -> Self {
		Self::Regular(Value::Nil)
	}
}


/// The call stack.
pub struct Stack(Vec<Slot>);


impl Stack {
	pub fn new() -> Self {
		Self(Vec::new())
	}


	/// Add elements to the top of the stack.
	pub fn extend(&mut self, slots: SlotIx) {
		let len = self.0.len();

		self.0.resize_with(len + slots.0 as usize, Slot::default);
	}


	/// Remove elements from the top of the stack.
	pub fn shrink(&mut self, slots: SlotIx) {
		self.0.truncate(self.0.len() - slots.0 as usize);
	}


	/// Fetch a value from the top of the stack.
	/// The offset is counted from the back.
	pub fn fetch(&self, slot_ix: SlotIx) -> Value {
		let vec = &self.0;
		let offset = slot_ix.0 as usize;

		vec[vec.len() - 1 - offset].fetch()
	}


	/// Capture a slot.
	pub fn capture(&mut self, slot_ix: SlotIx) -> Gc<GcCell<Value>> {
		let len = self.0.len();
		let vec = &mut self.0;
		let offset = slot_ix.0 as usize;

		vec[len - 1 - offset].capture()
	}


	/// Place a captured slot.
	pub fn place(&mut self, slot_ix: SlotIx, value: Gc<GcCell<Value>>) {
		let len = self.0.len();
		let vec = &mut self.0;
		let offset = slot_ix.0 as usize;

		vec[len - 1 - offset].place(value)
	}


	/// Store a value in the top of the stack.
	/// The offset is counted from the back.
	pub fn store(&mut self, slot_ix: SlotIx, value: Value) {
		let len = self.0.len();
		let vec = &mut self.0;
		let offset = slot_ix.0 as usize;

		vec[len - 1 - offset].store(value)
	}
}


impl Debug for Stack {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for (ix, val) in self.0.iter().rev().enumerate() {
			writeln!(f, "{}: {}", ix, val.fetch())?;
		}

		Ok(())
  }
}
