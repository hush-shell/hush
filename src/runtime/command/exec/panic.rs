use crate::{
	io::FileDescriptor,
	term::color,
};
use super::SourcePos;

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
