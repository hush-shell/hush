use std::convert::TryFrom;

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

	// Literal expansions:
	Home, // ~/
	Range(i64, i64), // {x..y}
	Collection(Box<[ArgUnit]>), // {a,b,c}

	// File expansions:
	Star, // *
	Percent, // %
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


/// Built-in commands.
#[derive(Debug, Copy, Clone)]
pub enum Builtin {
	Alias,
	Cd,
	Exec,
	Exec0,
	Spawn0,
}


#[derive(Debug)]
pub struct InvalidBuiltin;


impl std::error::Error for InvalidBuiltin { }


impl<'a> TryFrom<&'a [u8]> for Builtin {
	type Error = InvalidBuiltin;

	fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
		match value {
			b"alias" => Ok(Self::Alias),
			b"cd" => Ok(Self::Cd),
			b"exec" => Ok(Self::Exec),
			b"exec0" => Ok(Self::Exec0),
			b"spawn0" => Ok(Self::Spawn0),
			_ => Err(InvalidBuiltin)
		}
	}
}


impl<'a> TryFrom<&'a ast::Argument> for Builtin {
	type Error = InvalidBuiltin;

	fn try_from(arg: &'a ast::Argument) -> Result<Self, Self::Error> {
		match arg.parts.as_ref() {
			[ ast::ArgPart::Unit(ast::ArgUnit::Literal(ref lit)) ] => Self::try_from(lit.as_ref()),
			_ => Err(InvalidBuiltin),
		}
	}
}


/// A single command, including possible redirections and try operator.
#[derive(Debug)]
pub struct BasicCommand {
	pub program: Argument,
	/// Key-value pairs of environment variables.
	pub env: Box<[(ArgUnit, Argument)]>,
	pub arguments: Box<[Argument]>,
	pub redirections: Box<[Redirection]>,
	pub abort_on_error: bool,
	pub pos: SourcePos,
}


/// Commands may be pipelines, or a single BasicCommand.
#[derive(Debug)]
pub enum Command {
	Builtin {
		program: Builtin,
		arguments: Box<[Argument]>,
		abort_on_error: bool,
		pos: SourcePos,
	},
	External {
		head: BasicCommand,
		tail: Box<[BasicCommand]>
	}
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
