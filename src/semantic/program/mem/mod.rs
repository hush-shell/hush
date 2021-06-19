mod fmt;

use super::lexer;


/// The index of a memory slot in the activation record.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SlotIx(pub u32);


/// The kind of memory slot for a variable.
/// Whenever there is a `Closed` slot, there should be one or more corresponding `Capture`
/// slots in child functions.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SlotKind {
	/// Regular variables should be initialized as plain values.
	Regular,
	/// Closed-over variables should be initialized as boxed values.
	Closed,
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
	/// A memory slot for each variable.
	pub slots: Box<[SlotKind]>,
	/// Captured values from parent scope.
	pub captures: Box<[Capture]>,
	/// Where to insert `self`.
	pub self_slot: Option<SlotIx>,
}
