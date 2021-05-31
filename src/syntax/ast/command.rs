use crate::{io::FileDescriptor, symbol::Symbol};
use super::{lexer, SourcePos};


/// The most basic part of an argument.
pub enum ArgumentPart {
	Literal(Box<[u8]>),
	Dollar(Symbol),
}


/// An argument may consist of several argument parts.
pub struct Argument {
	pub parts: Box<[ArgumentPart]>,
	pub pos: SourcePos,
}


impl Argument {
	pub fn from_arg_parts(parts: Box<[lexer::ArgPart]>, pos: SourcePos) -> Self {
		enum Iter {
			Once(std::iter::Once<lexer::ArgUnit>),
			Vec(std::vec::IntoIter<lexer::ArgUnit>),
		}

		impl From<lexer::ArgPart> for Iter {
			fn from(part: lexer::ArgPart) -> Self {
				match part {
					lexer::ArgPart::Unquoted(unit) => Self::Once(std::iter::once(unit)),
					lexer::ArgPart::SingleQuoted(lit) => Self::Once(
						std::iter::once(lexer::ArgUnit::Literal(lit))
					),
					lexer::ArgPart::DoubleQuoted(units) => Self::Vec(units.into_vec().into_iter()),
				}
			}
		}

		impl Iterator for Iter {
			type Item = lexer::ArgUnit;

			fn next(&mut self) -> Option<Self::Item> {
				match self {
					Self::Once(iter) => iter.next(),
					Self::Vec(iter) => iter.next(),
				}
			}
		}

		let units = parts
			.into_vec()
			.into_iter()
			.flat_map(Into::<Iter>::into);

		let mut parts = Vec::<ArgumentPart>::with_capacity(1); // Expect at least one part.
		let mut literal = Option::<Vec<u8>>::None; // The accumulated literal, if any.

		for unit in units {
			match unit {
				lexer::ArgUnit::Literal(lit) => {
					if let Some(literal) = &mut literal {
						literal.extend(lit.iter());
					} else {
						literal = Some(lit.into());
					}
				}

				lexer::ArgUnit::Dollar(id) => {
					if let Some(literal) = literal.take() {
						parts.push(ArgumentPart::Literal(literal.into()));
					}

					parts.push(ArgumentPart::Dollar(id));
				}
			}
		}

		if let Some(lit) = literal {
			parts.push(ArgumentPart::Literal(lit.into()));
		}

		Argument {
			parts: parts.into(),
			pos
		}
	}
}


/// The target of a redirection operation.
pub enum RedirectionTarget {
	/// A numeric file descriptor.
	Fd(FileDescriptor),
	/// A file path.
	File(Argument),
}


/// Redirection operation.
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
