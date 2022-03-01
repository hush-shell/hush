use std::{
	io,
	collections::HashMap,
};

use crate::{
	io::FileDescriptor,
	term::color,
	symbol, runtime::value::{self, Value}, fmt::Show,
};
use super::{SourcePos, ErrorStatus};

/// A panic is an irrecoverable error in Hush.
#[derive(Debug)]
pub enum Panic {
	/// Expansion resulted in zero or multiple items where a single item was expected.
	InvalidArgs {
		object: &'static str,
		items: u32,
		pos: SourcePos,
	},
	/// Redirection of the given file descriptor is currently unsupported.
	UnsupportedFileDescriptor {
		fd: FileDescriptor,
		pos: SourcePos,
	},
}


impl Panic {
	/// Expansion resulted in zero or multiple items where a single item was expected.
	pub fn invalid_args(object: &'static str, items: u32, pos: SourcePos) -> Self {
		Self::InvalidArgs { object, items, pos }
	}

	/// Redirection of the given file descriptor is currently unsupported.
	pub fn unsupported_fd(fd: FileDescriptor, pos: SourcePos) -> Self {
		Self::UnsupportedFileDescriptor { fd, pos }
	}
}


/// We need this in order to be able to implement std::error::Error.
impl std::fmt::Display for Panic {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let panic = color::Fg(color::Red, "Panic");

		match self {
			Self::InvalidArgs { object, items, .. } =>
				write!(
					f,
					"{}: {} expansion resulted in {} items",
					panic,
					object,
					items
				),

			Self::UnsupportedFileDescriptor { fd, .. } =>
				write!(
					f,
					"{}: unsupported file descriptor ({})",
					panic,
					color::Fg(color::Yellow, fd)
				),
		}
	}
}


impl std::error::Error for Panic { }


impl Into<crate::runtime::Panic> for Panic {
	fn into(self) -> crate::runtime::Panic {
		use crate::runtime::Panic as P;

		match self {
			Panic::InvalidArgs { object, items, pos } => P::invalid_command_args(object, items, pos),
			Panic::UnsupportedFileDescriptor { fd, pos } => P::unsupported_fd(fd, pos),
		}
	}
}

pub trait IntoValue {
	fn into_value(self, interner: & symbol::Interner) -> Value;
}

impl IntoValue for ErrorStatus {
	fn into_value(mut self, interner: & symbol::Interner) -> Value {
		thread_local! {
			pub static STATUS: Value = "status".into();
			pub static POS: Value = "pos".into();
		}

		let description = std::mem::take(&mut self.description).into();

		let mut context = HashMap::new();
		STATUS.with(
			|status| context.insert(status.copy(), Value::Int(self.status as i64))
		);
		POS.with(
			|pos| context.insert(pos.copy(), Show(self.pos, interner).to_string().into())
		);
		let context = value::Dict::new(context).into();

		value::Error::new(description, context).into()
	}
}

#[derive(Debug)]
pub enum Error {
	Io {
		error: io::Error,
		pos: SourcePos,
	},
	Panic(Panic),
}


impl Error {
	pub fn io(error: io::Error, pos: SourcePos) -> Self {
		Self::Io { error, pos }
	}
}


impl From<Panic> for Error {
	fn from(panic: Panic) -> Self {
		Self::Panic(panic)
	}
}


/// Execution status of a pipeline.
#[derive(Debug)]
pub struct PipelineErrors(Box<[ErrorStatus]>);


impl PipelineErrors {
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

impl IntoValue for PipelineErrors {
	fn into_value(self, interner: & symbol::Interner) -> Value {
		let mut iter = self.0
			.into_vec() // Use vec's owned iterator.
			.into_iter()
			.map(|status| status.into_value(interner));

		let first = match iter.next() {
			None => return Value::default(),
			Some(error) => error,
		};

		if iter.len() == 0 {
			first
		} else {
			let mut errors = vec![first];
			errors.extend(iter);
			value::Error::new(
				"Some commands failed in the pipeline".into(),
				errors.into()
			).into()
		}
	}
}

impl IntoValue for Box<[PipelineErrors]> {
	fn into_value(self, interner: & symbol::Interner) -> Value {
		let mut iter = self
			.into_vec() // Use vec's owned iterator.
			.into_iter()
			.map(|error| error.into_value(interner));

		let first = match iter.next() {
			None => return Value::default(),
			Some(error) => error,
		};

		if iter.len() == 0 {
			first
		} else {
			let mut errors = vec![first];
			errors.extend(iter);
			value::Error::new(
				"Some commands failed in the block".into(),
				errors.into()
			).into()
		}
	}
}


impl From<ErrorStatus> for PipelineErrors {
	fn from(error: ErrorStatus) -> Self {
		Self([error].into())
	}
}


impl From<Option<ErrorStatus>> for PipelineErrors {
	fn from(error: Option<ErrorStatus>) -> Self {
		match error {
			Some(error) => error.into(),
			None => Self(Default::default()),
		}
	}
}


impl From<Vec<ErrorStatus>> for PipelineErrors {
	fn from(errors: Vec<ErrorStatus>) -> Self {
		Self(errors.into())
	}
}
