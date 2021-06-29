use std::{
	fmt::{self, Debug, Display},
	ops::Deref,
};

use gc::{Gc, GcCell, Finalize, Trace};

use super::{program, Value};


/// The index of a memory slot in the stack, relative to the top.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[derive(Trace, Finalize)]
pub struct SlotIx(pub u32);


impl SlotIx {
	/// Shallow copy.
	pub fn copy(&self) -> Self {
		Self(self.0)
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


/// A memory slot in the stack.
#[derive(Debug)]
enum Slot {
	/// A regular value.
	Regular(Value),
	/// A value that is closed over.
	Closed(Gc<GcCell<Value>>),
}


impl Slot {
	/// Fetch the value from the slot.
	fn fetch(&self) -> Value {
		match self {
			Slot::Regular(value) => value.copy(),
			Slot::Closed(value) => value.deref().borrow().copy(),
		}
	}


	/// Store a new value in the slot.
	fn store(&mut self, value: Value) {
		match self {
			Self::Closed(val) => *val.borrow_mut() = value,
			Self::Regular(val) => *val = value,
		}
	}


	/// Promote the slot to closed, returning the resulting garbage collected cell.
	/// This is a no-op if the slot is already closed.
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


	/// Place a garbage collected cell as a closed value.
	fn place(&mut self, value: Gc<GcCell<Value>>) {
		*self = Slot::Closed(value);
	}
}


impl Default for Slot {
	/// A default slot is a regular Nil value.
	fn default() -> Self {
		Self::Regular(Value::default())
	}
}


/// The call stack.
pub struct Stack {
	/// The stack of slots.
	slots: Vec<Slot>,
	/// The maximum allowed size for the stack.
	max_size: usize,
}


impl Stack {
	/// Create a new stack with the given maximum size.
	pub fn new(max_size: usize) -> Self {
		Self {
			slots: Vec::new(),
			max_size,
		}
	}


	/// Add the given ammount of Nil valued slots to the top of the stack.
	/// Returns StackOverflow if the size exceeds the maximum size.
	pub fn extend(&mut self, slots: SlotIx) -> Result<(), StackOverflow> {
		if self.len() > self.max_size {
			Err(StackOverflow)
		} else {
			// println!("extending stack by {}", slots.0);
			self.slots.resize_with(
				self.len() + slots.0 as usize,
				Slot::default
			);
			Ok(())
		}
	}


	/// Remove the given ammount of elements from the top of the stack.
	pub fn shrink(&mut self, slots: SlotIx) {
		self.slots.truncate(self.len() - slots.0 as usize);
	}


	/// Fetch the value of a slot.
	/// The offset is counted from the top.
	pub fn fetch(&self, slot_ix: SlotIx) -> Value {
		let offset = slot_ix.0 as usize;
		let value = self.slots[self.len() - 1 - offset].fetch();
		// println!("fetch #{}: {}", offset, value);
		value
	}


	/// Capture a slot.
	pub fn capture(&mut self, slot_ix: SlotIx) -> Gc<GcCell<Value>> {
		let len = self.len();
		let offset = slot_ix.0 as usize;
		// println!("capturing #{}", offset);
		self.slots[len - 1 - offset].capture()
	}


	/// Place a captured slot.
	pub fn place(&mut self, slot_ix: SlotIx, value: Gc<GcCell<Value>>) {
		let len = self.len();
		let offset = slot_ix.0 as usize;
		// println!("placing #{}: {}", offset, value.deref().borrow());
		self.slots[len - 1 - offset].place(value)
	}


	/// Store a value in a slot.
	/// The offset is counted from the top.
	pub fn store(&mut self, slot_ix: SlotIx, value: Value) {
		let len = self.len();
		let offset = slot_ix.0 as usize;
		// println!("storing #{}: {}", offset, value);
		self.slots[len - 1 - offset].store(value)
	}


	/// Check if the stack is empty.
	pub fn is_empty(&self) -> bool {
		self.slots.is_empty()
	}


	/// Get the stack length.
	fn len(&self) -> usize {
		self.slots.len()
	}
}


impl Default for Stack {
	fn default() -> Self {
		/// The default stack size for Hush programs.
		/// As the C stack is about 8MB by default, here is a naive correspondence.
		const STACK_SIZE: usize = (8 * 1024 * 1024) / std::mem::size_of::<Slot>();

		Self::new(STACK_SIZE)
	}
}


impl Debug for Stack {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (ix, val) in self.slots.iter().rev().enumerate() {
			writeln!(f, "{}: {}", ix, val.fetch())?;
		}

		Ok(())
	}
}


/// Stack overflow error.
#[derive(Debug)]
pub struct StackOverflow;


impl Display for StackOverflow {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "stack overflow")
  }
}


impl std::error::Error for StackOverflow { }
