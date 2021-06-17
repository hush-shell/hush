mod fmt;

use super::lexer;


/// The kind of memory slot for a variable.
/// Whenever there is a `Closed` slot, there should be one or more corresponding `Capture`
/// slots in child functions.
#[derive(Debug)]
pub enum SlotKind {
	/// Regular variables should be initialized as plain values.
	Regular,
	/// Closed-over variables should be initialized as boxed values.
	Closed,
	/// A variable inserted from a closed-over variable. Should be copied from the
	/// correspondent Closed variable, which is a boxed value.
	Capture { from: SlotIx },
}


/// The index of a memory slot in the activation record.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SlotIx(pub u32);


/// The mold for a stack frame.
#[derive(Debug)]
pub struct FrameInfo {
	/// A memory slot for each variable.
	pub slots: Box<[SlotKind]>,
}
