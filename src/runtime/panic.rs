use std::{borrow::Cow, io, ffi::OsString};

use crate::{
	fmt::{self, Display},
	io::FileDescriptor,
	term::color,
	symbol::{self, Symbol},
};
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
	/// Attempt to pop from empty collection.
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
		expected: Cow<'static, str>,
		pos: SourcePos,
	},
	/// Unexpected type.
	ValueError {
		value: Value,
		message: Cow<'static, str>,
		pos: SourcePos,
	},
	/// Attempt to assign a readonly field value.
	AssignToReadonlyField {
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
	/// Redirection of the given file descriptor is currently unsupported.
	UnsupportedFileDescriptor {
		fd: FileDescriptor,
		pos: SourcePos,
	},
	/// Currently, Hush requires patterns to be valid UTF-8.
	InvalidPattern {
		pattern: OsString,
		pos: SourcePos,
	},
	/// Assertion failed.
	AssertionFailed { pos: SourcePos },
	/// Failed to import module.
	ImportFailed {
		pos: SourcePos,
		path: Symbol,
	},
	/// Attempt to call <command>.join more than once.
	InvalidJoin { pos: SourcePos }
}


impl Panic {
	/// Attempt to increase the stack past it's maximum size.
	pub fn stack_overflow(pos: SourcePos) -> Self {
		Self::StackOverflow { pos }
	}


	/// Assertion failed.
	pub fn assertion_failed(pos: SourcePos) -> Self {
		Self::AssertionFailed { pos }
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


	/// Attempt to pop from empty collection.
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
	pub fn type_error<E>(value: Value, expected: E, pos: SourcePos) -> Self
	where
		E: Into<Cow<'static, str>>,
	{
		Self::TypeError {
			value,
			expected: expected.into(),
			pos,
		}
	}


	/// Invalid value.
	pub fn value_error<E>(value: Value, message: E, pos: SourcePos) -> Self
	where
		E: Into<Cow<'static, str>>,
	{
		Self::ValueError {
			value,
			message: message.into(),
			pos,
		}
	}


	/// Expansion resulted in zero or multiple items where a single item was expected.
	pub fn invalid_command_args(object: &'static str, items: u32, pos: SourcePos) -> Self {
		Self::InvalidCommandArgs { object, items, pos }
	}


	/// IO error.
	pub fn io(error: io::Error, pos: SourcePos) -> Self {
		Self::Io { error, pos }
	}


	/// Redirection of the given file descriptor is currently unsupported.
	pub fn unsupported_fd(fd: FileDescriptor, pos: SourcePos) -> Self {
		Self::UnsupportedFileDescriptor { fd, pos }
	}

	/// Currently, Hush requires patterns to be valid UTF-8.
	pub fn invalid_pattern(pattern: OsString, pos: SourcePos) -> Self {
		Self::InvalidPattern { pattern, pos }
	}


	/// Attempt to assign a readonly field value.
	pub fn assign_to_readonly_field(field: Value, pos: SourcePos) -> Self {
		Self::AssignToReadonlyField { field, pos }
	}

	/// Failed to import module.
	pub fn import_failed(path: Symbol, pos: SourcePos) -> Self {
		Self::ImportFailed { path, pos }
	}

	/// Attempt to call <command>.join more than once.
	pub fn invalid_join(pos: SourcePos) -> Self {
		Self::InvalidJoin { pos }
	}
}


impl<'a> Display<'a> for Panic {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		let panic = color::Fg(color::Red, "Panic");

		match self {
			Self::StackOverflow { pos } =>
				write!(f, "{} in {}: stack overflow", panic, fmt::Show(pos, context)),

			Self::IntegerOverflow { pos } =>
				write!(f, "{} in {}: integer overflow", panic, fmt::Show(pos, context)),

			Self::DivisionByZero { pos } =>
				write!(f, "{} in {}: division by zero", panic, fmt::Show(pos, context)),

			Self::IndexOutOfBounds { index, pos } =>
				write!(
					f,
					"{} in {}: index ({}) out of bounds",
					panic,
					fmt::Show(pos, context),
					color::Fg(color::Yellow, fmt::Show(index, context))
				),

			Self::EmptyCollection { pos } =>
				write!(f, "{} in {}: collection is empty", panic, fmt::Show(pos, context)),

			Self::InvalidCall { function, pos } =>
				write!(
					f,
					"{} in {}: attempt to call ({}), which is not a function",
					panic,
					fmt::Show(pos, context),
					color::Fg(color::Yellow, fmt::Show(function, context))
				),

			Self::InvalidArgs { supplied, expected, pos } =>
				write!(
					f,
					"{} in {}: incorrect amount of function parameters -- supplied {}, expected {}",
					panic,
					fmt::Show(pos, context),
					supplied,
					expected
				),

			Self::InvalidCondition { value, pos } =>
				write!(
					f,
					"{} in {}: condition ({}) is not a boolean",
					panic,
					fmt::Show(pos, context),
					color::Fg(color::Yellow, fmt::Show(value, context))
				),

			Self::TypeError { value, expected, pos } =>
				write!(
					f,
					"{} in {}: value ({}) has unexpected type, expected {}",
					panic,
					fmt::Show(pos, context),
					color::Fg(color::Yellow, fmt::Show(value, context)),
					expected,
				),

			Self::ValueError { value, message, pos } =>
				write!(
					f,
					"{} in {}: invalid value ({}), expected {}",
					panic,
					fmt::Show(pos, context),
					color::Fg(color::Yellow, fmt::Show(value, context)),
					message,
				),

			Self::InvalidCommandArgs { object, items, pos } =>
				write!(
					f,
					"{} in {}: {} expansion resulted in {} items",
					panic,
					fmt::Show(pos, context),
					object,
					items
				),

			Self::Io { error, pos } =>
				write!(f, "{} in {}: {}", panic, fmt::Show(pos, context), error),

			Self::UnsupportedFileDescriptor { fd, pos } =>
				write!(
					f,
					"{} in {}: unsupported file descriptor ({})",
					panic,
					fmt::Show(pos, context),
					color::Fg(color::Yellow, fd)
				),

			Self::InvalidPattern { pattern, pos } =>
				write!(
					f,
					"{} in {}: pattern ({:?}) has invalid UTF-8",
					panic,
					fmt::Show(pos, context),
					color::Fg(color::Yellow, pattern)
				),

			Self::AssignToReadonlyField { field, pos } => write!(
					f,
					"{} in {}: attempt to assign field ({}), which is readonly",
					panic,
					fmt::Show(pos, context),
					color::Fg(color::Yellow, fmt::Show(field, context))
				),

			Self::AssertionFailed { pos } =>
				write!(f, "{} in {}: assertion failed", panic, fmt::Show(pos, context)),

			Self::ImportFailed { path, pos } =>
				write!(
					f,
					"{} in {}: failed to import module ({})",
					panic,
					fmt::Show(pos, context),
					color::Fg(color::Yellow, fmt::Show(path, context))
				),

			Self::InvalidJoin { pos } =>
				write!(f, "{} in {}: attempt to call join more than once", panic, fmt::Show(pos, context)),
		}
	}
}


/// We need this in order to be able to implement std::error::Error.
impl std::fmt::Display for Panic {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		Display::fmt(self, f, &symbol::Interner::new())
	}
}


impl std::error::Error for Panic { }
