use std::{
	cmp::Ordering,
	hash::{Hash, Hasher}
};

use gc::{Gc, GcCell, Finalize, Trace};

use super::{
	mem,
	program,
	Panic,
	SourcePos,
	Value,
};


/// A function object.
/// Ord is required in order to be able to have dicts as dict keys.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Trace, Finalize)]
pub enum Function {
	Hush(HushFun),
	Rust(RustFun),
}


impl From<HushFun> for Function {
	fn from(fun: HushFun) -> Self {
		Self::Hush(fun)
	}
}


impl From<RustFun> for Function {
	fn from(fun: RustFun) -> Self {
		Self::Rust(fun)
	}
}


/// A function object implemented in Hush code.
/// May contain captured variables.
#[derive(Debug, Clone)]
#[derive(Trace, Finalize)]
pub struct HushFun {
	/// How many parameters the function expects.
	pub params: u32,
	pub frame_info: &'static program::mem::FrameInfo,
	pub body: &'static program::Block,
	/// Captured variables, if any.
	pub context: Box<[(Gc<GcCell<Value>>, mem::SlotIx)]>,
	pub pos: SourcePos,
}


impl PartialEq for HushFun {
	fn eq(&self, other: &Self) -> bool {
		// As the functions are defined in the source code, two functions can't share the same
		// body.
		std::ptr::eq(self.body, other.body)
			&& self.context == other.context
	}
}


impl Eq for HushFun { }


impl PartialOrd for HushFun {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}


impl Ord for HushFun {
	fn cmp(&self, other: &Self) -> Ordering {
		// As the functions are defined in the source code, two functions can't share the same
		// source position.
		self.pos.cmp(&other.pos)
	}
}


impl Hash for HushFun {
	fn hash<H: Hasher>(&self, state: &mut H) {
		// As the functions are defined in the source code, two functions can't share the same
		// source position.
		self.pos.hash(state)
	}
}


/// A function object implemented in Rust.
#[derive(Clone)]
#[derive(Finalize)]
pub struct RustFun {
	/// The fully qualified name of the function. E.g.: std.print
	pub name: &'static str,
	/// The function implementation.
	/// Parameters:
	/// - A slice of arguments.
	/// - The source position of the call, which allows proper location of panics.
	pub fun: fn(&mut [Value], SourcePos) -> Result<Value, Panic>,
}

unsafe impl Trace for RustFun {
	gc::unsafe_empty_trace!();
}


impl PartialEq for RustFun {
	fn eq(&self, other: &Self) -> bool {
		let _self = self.fun as *const ();
		let _other = other.fun as *const ();
		std::ptr::eq(_self, _other)
	}
}


impl Eq for RustFun { }


impl PartialOrd for RustFun {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}


impl Ord for RustFun {
	fn cmp(&self, other: &Self) -> Ordering {
		let _self = self.fun as usize;
		let _other = other.fun as usize;
		_self.cmp(&_other)
	}
}


impl Hash for RustFun {
	fn hash<H: Hasher>(&self, state: &mut H) {
		let fun = self.fun as *const ();
		std::ptr::hash(fun, state)
	}
}
