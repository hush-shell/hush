use std::{
	io,
	fmt::{self, Display},
};

use crate::term::color;
use super::{Value, SourcePos};


/// A panic is an irrecoverable error in Hush.
#[derive(Debug)]
pub enum Panic {
	/// Attempt to increase the stack past it's maximum size.
	StackOverflow { pos: SourcePos },
	/// Integer overflow.
	IntegerOverflow { pos: SourcePos },
	/// Integer division by zero.
	DivisionByZero { pos: SourcePos },
	/// Array or dict index out of bounds.
	IndexOutOfBounds {
		index: Value,
		pos: SourcePos,
	},
	/// Array or dict index out of bounds.
	EmptyCollection { pos: SourcePos },
	/// Attempt to call a non-function value.
	InvalidCall {
		function: Value,
		pos: SourcePos
	},
	/// Ammount of supplied arguments in function call is different than expected.
	InvalidArgs {
		supplied: u32,
		expected: u32,
		pos: SourcePos
	},
	/// Conditional expression is not a boolean.
	InvalidCondition {
		value: Value,
		pos: SourcePos,
	},
	/// Unexpected type.
	TypeError {
		value: Value,
		pos: SourcePos,
	},
	/// Attempt to assign a field of an error value.
	AssignToErrorField {
		field: Value,
		pos: SourcePos,
	},
	/// Expansion resulted in zero or multiple items where a single item was expected.
	InvalidCommandArgs {
		object: &'static str,
		items: u32,
		pos: SourcePos,
	},
	/// IO error in the standard library.
	Io {
		error: io::Error,
		pos: SourcePos,
	},
}


impl Panic {
	/// Attempt to increase the stack past it's maximum size.
	pub fn stack_overflow(pos: SourcePos) -> Self {
		Self::StackOverflow { pos }
	}


	/// Integer division by zero.
	pub fn integer_overflow(pos: SourcePos) -> Self {
		Self::IntegerOverflow { pos }
	}


	/// Integer division by zero.
	pub fn division_by_zero(pos: SourcePos) -> Self {
		Self::DivisionByZero { pos }
	}


	/// Array or dict index out of bounds.
	pub fn index_out_of_bounds(index: Value, pos: SourcePos) -> Self {
		Self::IndexOutOfBounds { index, pos }
	}


	/// Array or dict index out of bounds.
	pub fn empty_collection(pos: SourcePos) -> Self {
		Self::EmptyCollection { pos }
	}


	/// Attempt to call a non-function value.
	pub fn invalid_call(function: Value, pos: SourcePos) -> Self {
		Self::InvalidCall { function, pos }
	}


	/// Ammount of supplied arguments in function call is different than expected.
	pub fn invalid_args(supplied: u32, expected: u32, pos: SourcePos) -> Self {
		Self::InvalidArgs { supplied, expected, pos }
	}


	/// Conditional expression is not a boolean.
	pub fn invalid_condition(value: Value, pos: SourcePos) -> Self {
		Self::InvalidCondition { value, pos }
	}


	/// Unexpected type.
	pub fn type_error(value: Value, pos: SourcePos) -> Self {
		Self::TypeError { value, pos }
	}


	/// Expansion resulted in zero or multiple items where a single item was expected.
	pub fn invalid_command_args(object: &'static str, items: u32, pos: SourcePos) -> Self {
		Self::InvalidCommandArgs { object, items, pos }
	}


	/// IO error.
	pub fn io(error: io::Error, pos: SourcePos) -> Self {
		Self::Io { error, pos }
	}


	/// Attempt to assign a field of an error value.
	pub fn assign_to_error_field(field: Value, pos: SourcePos) -> Self {
		Self::AssignToErrorField { field, pos }
	}
}


impl Display for Panic {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let panic = color::Fg(color::Red, "Panic");

		match self {
			Self::StackOverflow { pos } =>
				write!(f, "{} in {}: stack overflow", panic, pos),

			Self::IntegerOverflow { pos } =>
				write!(f, "{} in {}: integer overflow", panic, pos),

			Self::DivisionByZero { pos } =>
				write!(f, "{} in {}: division by zero", panic, pos),

			Self::IndexOutOfBounds { index, pos } =>
				write!(
					f,
					"{} in {}: index ({}) out of bounds",
					panic,
					pos,
					color::Fg(color::Yellow, index)
				),

			Self::EmptyCollection { pos } =>
				write!(f, "{} in {}: collection is empty", panic, pos),

			Self::InvalidCall { function, pos } =>
				write!(
					f,
					"{} in {}: attempt to call ({}), which is not a function",
					panic,
					pos,
					color::Fg(color::Yellow, function)
				),

			Self::InvalidArgs { supplied, expected, pos } =>
				write!(
					f,
					"{} in {}: incorrect ammount of function parameters -- supplied {}, expected {}",
					panic,
					pos,
					supplied,
					expected
				),

			Self::InvalidCondition { value, pos } =>
				write!(
					f,
					"{} in {}: condition ({}) is not a boolean",
					panic,
					pos,
					color::Fg(color::Yellow, value)
				),

			Self::TypeError { value, pos } =>
				write!(
					f,
					"{} in {}: value ({}) has unexpected type",
					panic,
					pos,
					color::Fg(color::Yellow, value)
				),

			Self::InvalidCommandArgs { object, items, pos } =>
				write!(f, "{} in {}: {} expansion resulted in {} items", panic, pos, object, items),

			Self::Io { error, pos } =>
				write!(f, "{} in {}: {}", panic, pos, error),

			Self::AssignToErrorField { field, pos } => write!(
					f,
					"{} in {}: attempt to assign field ({}) in an error value, which is immutable",
					panic,
					pos,
					color::Fg(color::Yellow, field)
				),
		}
	}
}


impl std::error::Error for Panic { }
