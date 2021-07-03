use std::{
	cmp::Ordering,
	fmt::{self, Debug},
	hash::{Hash, Hasher},
	ops::Deref,
};

use gc::{Gc, GcCell, Finalize, Trace};

use crate::symbol;
use super::{
	mem,
	program,
	Panic,
	Runtime,
	SourcePos,
	Value,
};


/// A function object.
/// Ord is required in order to be able to have dicts as dict keys.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Trace, Finalize)]
pub enum Function {
	Hush(HushFun),
	Rust(RustFun),
}


impl Function {
	/// Shallow copy.
	pub fn copy(&self) -> Self {
		match self {
			Function::Hush(fun) => Function::Hush(fun.copy()),
			Function::Rust(fun) => Function::Rust(fun.copy()),
		}
	}
}


impl From<HushFun> for Function {
	fn from(fun: HushFun) -> Self {
		Self::Hush(fun)
	}
}


impl<T: NativeFun> From<T> for Function {
	fn from(fun: T) -> Self {
		Self::Rust(fun.into())
	}
}


/// A function object implemented in Hush code.
/// May contain captured variables.
#[derive(Debug)]
#[derive(Trace, Finalize)]
pub struct HushFun {
	/// How many parameters the function expects.
	pub params: u32,
	pub frame_info: &'static program::mem::FrameInfo,
	pub body: &'static program::Block,
	/// Captured variables, if any.
	#[allow(clippy::type_complexity)]
	pub context: Gc<Box<[(Gc<GcCell<Value>>, mem::SlotIx)]>>,
	pub pos: SourcePos,
}


impl HushFun {
	pub fn new (
		params: u32,
		frame_info: &'static program::mem::FrameInfo,
		body: &'static program::Block,
		context: Box<[(Gc<GcCell<Value>>, mem::SlotIx)]>,
		pos: SourcePos,
	) -> Self {
		Self {
			params,
			frame_info,
			body,
			context: Gc::new(context),
			pos,
		}
	}


	/// Shallow copy.
	pub fn copy(&self) -> Self {
		Self {
			params: self.params,
			frame_info: self.frame_info,
			body: self.body,
			context: self.context.clone(),
			pos: self.pos.copy(),
		}
	}
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


/// Context for a native function call.
#[derive(Debug)]
pub struct CallContext<'a, 'b> {
	/// The runtime instance.
	pub runtime: &'b mut Runtime<'a>,
	/// The value of `self`.
	pub obj: Value,
	/// The offset in the runtime's argument vector where the arguments for this call are
	/// placed.
	pub args_start: usize,
	/// The source position of the call, which allows proper location of panics.
	pub pos: SourcePos,
}


impl<'a, 'b> CallContext<'a, 'b> {
	/// Get the slice of arguments.
	pub fn args(&self) -> &[Value] {
		&self.runtime.arguments[self.args_start..]
	}


	/// Get the mutable slice of arguments.
	pub fn args_mut(&mut self) -> &mut [Value] {
		&mut self.runtime.arguments[self.args_start..]
	}


	/// Get the symbol interner.
	pub fn interner(&self) -> & symbol::Interner {
		&self.runtime.interner
	}


	pub fn call(
		&mut self,
		obj: Value,
		function: &Function,
		args_start: usize,
	) -> Result<Value, Panic> {
		self.runtime.call(obj, function, args_start, self.pos.copy())
	}
}


/// A native function implementation.
pub trait NativeFun: Trace + Finalize + 'static {
	/// Get a human-readable name for the function.
	/// This is also used for equality, ordering and hashing, and therefore must be a
	/// globally unique name.
	fn name(&self) -> &'static str;
	/// Invoke the function.
	fn call(&mut self, context: CallContext) -> Result<Value, Panic>;
}


/// A garbage-collected native function.
#[derive(Trace, Finalize)]
pub struct RustFun(Gc<GcCell<Box<dyn NativeFun>>>);


impl RustFun {
	/// Shallow copy.
	pub fn copy(&self) -> Self {
		Self(self.0.clone())
	}


	/// Get a human-readable name for the function.
	pub fn name(&self) -> &'static str {
		self.0.deref().borrow().name()
	}


	/// Invoke the function.
	pub fn call(&self, context: CallContext) -> Result<Value, Panic> {
		self.0.deref().borrow_mut().call(context)
	}
}


impl Debug for RustFun {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.name())
	}
}


impl<T: NativeFun> From<T> for RustFun {
	fn from(fun: T) -> Self {
		Self(Gc::new(GcCell::new(Box::new(fun))))
	}
}


impl PartialEq for RustFun {
	fn eq(&self, other: &Self) -> bool {
		self.0.deref().borrow().name() == other.0.deref().borrow().name()
	}
}


impl Eq for RustFun { }


impl PartialOrd for RustFun {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.0.deref().borrow().name().partial_cmp(other.0.deref().borrow().name())
	}
}


impl Ord for RustFun {
	fn cmp(&self, other: &Self) -> Ordering {
		self.0.deref().borrow().name().cmp(other.0.deref().borrow().name())
	}
}


impl Hash for RustFun {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.deref().borrow().name().hash(state);
	}
}
