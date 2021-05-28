use crate::{io::FileDescriptor, symbol::Symbol};


/// The most basic part of an argument.
#[derive(Debug)]
enum ArgumentPart {
	Literal(Box<[u8]>),
	Dollar(Symbol),
}


/// An argument may consist of several argument parts.
#[derive(Debug)]
pub struct Argument {
	parts: Box<[ArgumentPart]>,
}


/// The target of a redirection operation.
#[derive(Debug)]
pub enum RedirectionTarget {
	/// A numeric file descriptor.
	Fd(FileDescriptor),
	/// A file path.
	File(Argument),
}


/// Redirection operation.
#[derive(Debug)]
pub enum Redirection {
	Output {
		source: FileDescriptor,
		target: RedirectionTarget,
	},
	OutputAppend {
		source: FileDescriptor,
		target: Argument,
	},
	Input {
		/// Whether the source is the input or the file path.
		literal: bool,
		source: Argument,
	},
}


/// A single command, including possible redirections and try operator.
#[derive(Debug)]
pub struct BasicCommand {
	abort_on_error: bool,
	arguments: Box<[Argument]>,
	redirections: Box<[Redirection]>,
}


/// Commands may be pipelines, or a single BasicCommand.
#[derive(Debug)]
pub struct Command(Box<[BasicCommand]>);


/// The kinds of command blocks.
#[derive(Debug)]
pub enum CommandBlockKind {
	Synchronous,  // {}
	Asynchronous, // &{}
	Capture,      // ${}
}
