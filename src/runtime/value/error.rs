use std::{
	io,
	hash::{Hash, Hasher},
	ops::Deref,
};

use gc::{Gc, GcCell, Finalize, Trace};

use super::{IndexOutOfBounds, Value, Str};


/// Strings in Hush are immutable.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Trace, Finalize)]
pub struct Error {
	pub description: Str,
	pub context: Gc<GcCell<Value>>,
}


impl Error {
	/// Create a new error instance.
	pub fn new(description: Str, context: Value) -> Self {
		Self {
			description,
			context: Gc::new(GcCell::new(context)),
		}
	}

	/// Shallow copy.
	pub fn copy(&self) -> Self {
		Self {
			description: self.description.copy(),
			context: self.context.clone(),
		}
	}


	/// Get the given property.
	pub fn get(&self, key: &Value) -> Result<Value, IndexOutOfBounds> {
		thread_local! {
			pub static DESCRIPTION: Value = "description".into();
			pub static CONTEXT: Value = "context".into();
		}

		match key {
			key if DESCRIPTION.with(|desc| key == desc) => Ok(
				self.description
					.copy()
					.into()
			),

			key if CONTEXT.with(|ctx| key == ctx) => Ok(
				self.context
					.deref()
					.borrow()
					.copy()
			),

			_ => Err(IndexOutOfBounds)
		}
	}
}


impl Hash for Error {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.description.hash(state);
		self.context.deref().borrow().hash(state);
	}
}


impl From<io::Error> for Error {
	fn from(error: io::Error) -> Self {
		Self::new(error.to_string().into(), Value::Nil)
	}
}
