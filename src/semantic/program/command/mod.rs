pub mod builtin;

use crate::io::FileDescriptor;
use super::{ast, mem, SourcePos};


/// The most basic part of an argument.
#[derive(Debug)]
pub enum ArgUnit {
	Literal(Box<[u8]>),
	Dollar {
		slot_ix: mem::SlotIx,
		pos: SourcePos,
	}
}


/// The most basic part of an argument.
#[derive(Debug)]
pub enum ArgPart {
	Unit(ArgUnit),

	// Expansions:
	Home, // ~/
	Range(i64, i64), // {x..y}
	Collection(Box<[ArgUnit]>), // {a,b,c}

	// Regex expansions:
	Star, // *
	Question, // ?
	CharClass(Box<[u8]>), // [...]
}


/// An argument may consist of several argument parts.
#[derive(Debug)]
pub struct Argument {
	pub parts: Box<[ArgPart]>,
	pub pos: SourcePos,
}


/// The target of a redirection operation.
#[derive(Debug)]
pub enum RedirectionTarget {
	/// Redirect to a file descriptor.
	Fd(FileDescriptor),
	/// Overwrite a file.
	Overwrite(Argument),
	/// Append to a file.
	Append(Argument),
}


/// Redirection operation.
#[derive(Debug)]
pub enum Redirection {
	/// Redirect output to a file or file descriptor.
	Output {
		source: FileDescriptor,
		target: RedirectionTarget,
	},
	/// Redirect input from a file or literal.
	Input {
		/// Whether the source is the input or the file path.
		literal: bool,
		source: Argument,
	},
}


/// A single command, including possible redirections and try operator.
#[derive(Debug)]
pub struct BasicCommand {
	pub program: Argument,
	pub arguments: Box<[Argument]>,
	pub redirections: Box<[Redirection]>,
	pub abort_on_error: bool,
	pub pos: SourcePos,
}


/// Commands may be pipelines, or a single BasicCommand.
#[derive(Debug)]
pub struct Command {
	pub head: BasicCommand,
	pub tail: Box<[BasicCommand]>
}


/// A command block.
#[derive(Debug)]
pub struct CommandBlock {
	pub kind: CommandBlockKind,
	pub head: Command,
	pub tail: Box<[Command]>,
}


/// The kinds of command blocks.
#[derive(Debug)]
pub enum CommandBlockKind {
	Synchronous,  // {}
	Asynchronous, // &{}
	Capture,      // ${}
}


impl From<ast::CommandBlockKind> for CommandBlockKind {
	fn from(kind: ast::CommandBlockKind) -> Self {
		match kind {
			ast::CommandBlockKind::Synchronous => CommandBlockKind::Synchronous,
			ast::CommandBlockKind::Asynchronous => CommandBlockKind::Asynchronous,
			ast::CommandBlockKind::Capture => CommandBlockKind::Capture,
		}
	}
}
