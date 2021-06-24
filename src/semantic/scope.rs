use std::collections::hash_map::{HashMap, Entry};

use crate::symbol::{self, Symbol};
use super::{
	mem::{Capture, FrameInfo, SlotIx},
	Error,
	SourcePos,
};


/// A lexical scope, which translates identifiers to slot indexes.
#[derive(Debug, Default)]
struct Scope {
	variables: HashMap<Symbol, SlotIx>,
}


impl Scope {
	/// Try to declare a variable in the current scope with the given index.
	/// If the variable is already declared, returns false.
	fn declare(&mut self, symbol: Symbol, ix: SlotIx) -> bool {
		match self.variables.entry(symbol) {
			Entry::Occupied(_) => false,

			Entry::Vacant(entry) => {
				entry.insert(ix);
				true
			}
		}
	}


	/// Resolve the index for an already declared variable.
	fn resolve(&mut self, symbol: Symbol) -> Option<SlotIx> {
		self.variables
			.get(&symbol)
			.copied()
	}
}


/// A function lexical scope.
/// Includes information about local variables.
#[derive(Debug)]
struct Frame {
	/// How many slots in the frame.
	slots: SlotIx,
	/// Captured slots.
	captures: Vec<Capture>,
	/// The slot index of `self`.
	self_slot: Option<SlotIx>,
	/// Stack of scopes in the frame.
	scopes: Vec<Scope>,
}


impl Frame {
	/// Create a new frame. The root scope must be entered manually.
	fn new() -> Self {
		Self {
			slots: SlotIx(0),
			captures: Vec::new(),
			self_slot: None,
			scopes: Vec::new(),
		}
	}


	/// Enter a new empty scope.
	fn enter_block(&mut self) {
		self.scopes.push(Scope::default());
	}


	/// Exit the current scope.
	/// Panics if the stack is empty.
	fn exit_block(&mut self) {
		self.scopes
			.pop()
			.expect("attempt to exit empty stack");
	}


	/// Declares a symbol in the current scope.
	/// Panics if the stack is empty.
	fn declare(&mut self, symbol: Symbol, pos: SourcePos) -> Result<SlotIx, Error> {
		let scope = self.scopes.last_mut().expect("attempt to declare in empty stack");

		if scope.declare(symbol, self.slots) {
			Ok(self.slots.bump())
		} else {
			Err(Error::duplicate_variable(symbol, pos))
		}
	}


	/// Resolve a symbol in the current frame.
	fn resolve(&mut self, symbol: Symbol) -> Option<SlotIx> {
		self.scopes
			.iter_mut()
			.rev()
			.find_map(
				|scope| scope.resolve(symbol)
			)
	}


	/// Capture a variable from a parent scope.
	/// If the variable was already captured, just returns the slot index.
	fn capture(&mut self, symbol: Symbol, parent_slot_ix: SlotIx) -> SlotIx {
		let scope = self.scopes.first_mut().expect("frame missing root scope");

		match scope.variables.entry(symbol) {
			Entry::Occupied(entry) => *entry.get(),

			Entry::Vacant(entry) => {
				let slot_ix = self.slots.bump();
				entry.insert(slot_ix);
				self.captures.push(
					Capture {
						from: parent_slot_ix,
						to: slot_ix,
					}
				);

				slot_ix
			}
		}
	}


	/// Resolve or insert a slot for `self`
	fn resolve_or_insert_self(&mut self) -> SlotIx {
		match self.self_slot {
			Some(slot_ix) => slot_ix,

			None => {
				let slot_ix = self.slots.bump();
				self.self_slot = Some(slot_ix);
				slot_ix
			}
		}
	}
}


impl Drop for Frame {
	fn drop(&mut self) {
		debug_assert!(self.scopes.is_empty())
	}
}


impl Into<FrameInfo> for Frame {
	fn into(mut self) -> FrameInfo {
		FrameInfo {
			slots: self.slots,
			captures: std::mem::take(&mut self.captures).into(),
			self_slot: self.self_slot,
		}
	}
}


/// A function scope stack.
#[derive(Debug, Default)]
pub struct Stack {
	frames: Vec<Frame>,
}


impl Stack {
	/// Enter a new empty frame.
	pub fn enter_frame(&mut self) {
		let mut frame = Frame::new();

		frame.enter_block();

		self.frames.push(frame);
	}


	/// Exit the current frame scope, returning the produced frame info.
	/// Panics if the stack is empty.
	pub fn exit_frame(&mut self) -> FrameInfo {
		let mut frame = self.frames
			.pop()
			.expect("attempt to exit empty stack");

		frame.exit_block();

		debug_assert!(frame.scopes.is_empty());

		frame.into()
	}


	/// Enter a new block in the current frame.
	pub fn enter_block(&mut self) {
		self.top().enter_block()
	}


	/// Exit a new block in the current frame.
	pub fn exit_block(&mut self) {
		self.top().exit_block()
	}


	/// Declares a symbol in the current scope.
	/// Panics if the stack is empty.
	pub fn declare(&mut self, symbol: Symbol, pos: SourcePos) -> Result<SlotIx, Error> {
		self.top().declare(symbol, pos)
	}


	/// Resolve a symbol in the current scope.
	/// If the symbol is being closed from a parent frame, the capturing of such symbol is
	/// installed.
	pub fn resolve(
		&mut self,
		symbol: Symbol,
		pos: SourcePos,
		interner: &mut symbol::Interner,
	) -> Result<SlotIx, Error> {
		// Resolve the frame and slot where the symbol originates from.
		let (frame_ix, mut slot_ix) = self.frames
			.iter_mut()
			.enumerate()
			.rev()
			.find_map(
				|(frame_ix, frame)| {
					let slot_ix = frame.resolve(symbol)?;
					Some((frame_ix, slot_ix))
				}
			)
			.ok_or_else(
				|| Error::undeclared_variable(symbol, pos)
			)?;

		if frame_ix == self.frames.len() - 1 { // Symbol found in the local frame.
			return Ok(slot_ix)
		} else { // Symbol captured from parent frame, must setup capturing.
			let symbol_captured = {
				// Create a new unique symbol for the intermediate local variables. Note that this
				// identifier contains an invalid character, and therefore can not clash with
				// user-defined identifiers.
				let mut identifier = interner
					.resolve(symbol)
					.expect("unresolved symbol")
					.to_owned();

				identifier.push_str("@closed");

				interner.get_or_intern(identifier)
			};

			// Insert the captured slot in the intermediate frames between the origin and the
			// destination.
			let range = frame_ix + 1 .. self.frames.len();

			for frame in &mut self.frames[range] {
				slot_ix = frame.capture(symbol_captured, slot_ix);
			}
		}

		Ok(slot_ix)
	}


	/// Resolve or insert a slot for `self`
	pub fn resolve_or_insert_self(&mut self) -> SlotIx {
		self.top().resolve_or_insert_self()
	}


	/// Get the top frame in the stack.
	fn top(&mut self) -> &mut Frame {
		self.frames.last_mut().expect("empty stack")
	}
}


impl Drop for Stack {
	fn drop(&mut self) {
		debug_assert!(self.frames.is_empty())
	}
}
