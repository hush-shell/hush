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
	/// Integer division by zero.
	DivisionByZero { pos: SourcePos },
	/// Array or dict index out of bounds.
	IndexOutOfBounds {
		index: Value,
		pos: SourcePos,
	},
	/// Attempt to call a non-function value.
	InvalidCall {
		function: Value,
		pos: SourcePos
	},
	/// Missing parameters in function call.
	MissingParameters { pos: SourcePos },
	/// Conditional expression is not a boolean.
	InvalidCondition {
		value: Value,
		pos: SourcePos,
	},
	/// Invalid type for operator.
	InvalidOperand {
		value: Value,
		pos: SourcePos,
	},
	/// IO error in the standard library.
	Io(io::Error),
}


impl Panic {
	/// Attempt to increase the stack past it's maximum size.
	pub fn stack_overflow(pos: SourcePos) -> Self {
		Self::StackOverflow { pos }
	}


	/// Integer division by zero.
	pub fn division_by_zero(pos: SourcePos) -> Self {
		Self::DivisionByZero { pos }
	}


	/// Array or dict index out of bounds.
	pub fn index_out_of_bounds(index: Value, pos: SourcePos) -> Self {
		Self::IndexOutOfBounds { index, pos }
	}


	/// Attempt to call a non-function value.
	pub fn invalid_call(function: Value, pos: SourcePos) -> Self {
		Self::InvalidCall { function, pos }
	}


	/// Missing parameters in function call.
	pub fn missing_parameters(pos: SourcePos) -> Self {
		Self::MissingParameters { pos }
	}


	/// Conditional expression is not a boolean.
	pub fn invalid_condition(value: Value, pos: SourcePos) -> Self {
		Self::InvalidCondition { value, pos }
	}


	/// Invalid type for operator.
	pub fn invalid_operand(value: Value, pos: SourcePos) -> Self {
		Self::InvalidOperand { value, pos }
	}
}


impl From<io::Error> for Panic {
	fn from(error: io::Error) -> Self {
		Self::Io(error)
	}
}


impl Display for Panic {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::StackOverflow { pos } =>
				write!(f, "{} in {}: stack overflow", color::Fg(color::Red, "Panic"), pos),

			Self::DivisionByZero { pos } =>
				write!(f, "{} in {}: division by zero", color::Fg(color::Red, "Panic"), pos),

			Self::IndexOutOfBounds { index, pos } =>
				write!(
					f,
					"{} in {}: index ({}) out of bounds",
					color::Fg(color::Red, "Panic"),
					pos,
					color::Fg(color::Yellow, index)
				),

			Self::InvalidCall { function, pos } =>
				write!(
					f,
					"{} in {}: attempt to call ({}), which is not a function",
					color::Fg(color::Red, "Panic"),
					pos,
					color::Fg(color::Yellow, function)
				),

			Self::MissingParameters { pos } =>
				write!(f, "{} in {}: missing function parameters", color::Fg(color::Red, "Panic"), pos),

			Self::InvalidCondition { value, pos } =>
				write!(
					f,
					"{} in {}: condition ({}) is not a boolean",
					color::Fg(color::Red, "Panic"),
					pos,
					color::Fg(color::Yellow, value)
				),

			Self::InvalidOperand { value, pos } =>
				write!(
					f,
					"{} in {}: operand ({}) has an invalid type",
					color::Fg(color::Red, "Panic"),
					pos,
					color::Fg(color::Yellow, value)
				),

			Self::Io(error) =>
				write!(f, "{}: {}", color::Fg(color::Red, "Panic"), error),
		}
	}
}


impl std::error::Error for Panic { }
