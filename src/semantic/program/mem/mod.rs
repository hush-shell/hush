mod fmt;

use super::lexer;


/// The index of a memory slot in the activation record.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SlotIx(pub u32);


impl SlotIx {
	pub(in crate::semantic) fn bump(&mut self) -> SlotIx {
		let previous = *self;
		self.0 += 1;
		previous
	}
}


/// How to capture a value from the parent scope.
#[derive(Debug, Copy, Clone)]
pub struct Capture {
	/// The slot index in the parent scope.
	pub from: SlotIx,
	/// The slot index in our scope.
	pub to: SlotIx,
}


/// The mold for a stack frame.
#[derive(Debug)]
pub struct FrameInfo {
	/// How many slots in the activation record.
	pub slots: SlotIx,
	/// Captured values from parent scope.
	pub captures: Box<[Capture]>,
	/// Where to insert `self`.
	pub self_slot: Option<SlotIx>,
}
