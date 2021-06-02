use crate::{io::FileDescriptor, symbol::Symbol};
use super::{lexer, SourcePos};


/// The most basic part of an argument.
pub enum ArgUnit {
	Literal(Box<[u8]>),
	Dollar(Symbol),
}


/// The most basic part of an argument.
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
pub struct Argument {
	pub parts: Box<[ArgPart]>,
	pub pos: SourcePos,
}


/// The target of a redirection operation.
pub enum RedirectionTarget {
	/// Redirect to a file descriptor.
	Fd(FileDescriptor),
	/// Overwrite a file.
	Overwrite(Argument),
	/// Append to a file.
	Append(Argument),
}


/// Redirection operation.
pub enum Redirection {
	Output {
		source: FileDescriptor,
		target: RedirectionTarget,
	},
	Input {
		/// Whether the source is the input or the file path.
		literal: bool,
		source: Argument,
	},
}


/// A single command, including possible redirections and try operator.
pub struct BasicCommand {
	pub command: Argument,
	pub arguments: Box<[Argument]>,
	pub redirections: Box<[Redirection]>,
	pub abort_on_error: bool,
	pub pos: SourcePos,
}


/// Commands may be pipelines, or a single BasicCommand.
pub struct Command(pub Box<[BasicCommand]>);


impl From<Box<[BasicCommand]>> for Command {
	fn from(commands: Box<[BasicCommand]>) -> Self {
		Self(commands)
	}
}


/// The kinds of command blocks.
pub enum CommandBlockKind {
	Synchronous,  // {}
	Asynchronous, // &{}
	Capture,      // ${}
}


impl CommandBlockKind {
	pub fn from_token(token: &lexer::TokenKind) -> Option<Self> {
		match token {
			lexer::TokenKind::Command => Some(Self::Synchronous),
			lexer::TokenKind::AsyncCommand => Some(Self::Asynchronous),
			lexer::TokenKind::CaptureCommand => Some(Self::Capture),
			_ => None,
		}
	}
}
