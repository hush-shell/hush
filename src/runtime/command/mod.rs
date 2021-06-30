mod arg;
mod fmt;

use std::{
	borrow::Cow,
	ffi::OsStr,
	os::unix::ffi::OsStrExt,
	process::{self, Child},
};

use regex::bytes::Regex;

use crate::io::FileDescriptor;
use super::{program, Runtime, Panic, Value, SourcePos};
use arg::Args;


/// An argument may expand to zero or more literals.
#[derive(Debug)]
enum Argument {
	/// A regex to be matched to file names. May expand to zero or more literals.
	Regex(Regex),
	/// A single literal.
	Literal(Box<OsStr>),
}


/// The target of a redirection operation.
#[derive(Debug)]
enum RedirectionTarget {
	/// Redirect to a file descriptor.
	Fd(FileDescriptor),
	/// Overwrite a file. Panics if the argument does not expand to a single literal.
	Overwrite(Argument),
	/// Append to a file. Panics if the argument does not expand to a single literal.
	Append(Argument),
}


/// Redirection operation.
#[derive(Debug)]
enum Redirection {
	/// Redirect output to a file or file descriptor.
	Output {
		source: FileDescriptor,
		target: RedirectionTarget,
	},
	/// Redirect input from a file or literal.
	Input {
		/// Whether the source is the input or the file path.
		literal: bool,
		/// The source argument. Panics if the argument does not expand to a single literal.
		source: Argument,
	},
}


/// Built-in commands.
#[derive(Debug, Copy, Clone)]
enum Builtin {
	Alias,
	Cd,
}


impl<'a> From<&'a program::command::Builtin> for Builtin {
	fn from(builtin: &'a program::command::Builtin) -> Self {
		match builtin {
			program::command::Builtin::Alias => Self::Alias,
			program::command::Builtin::Cd => Self::Cd,
		}
	}
}


/// A single command, including possible redirections and try operator.
#[derive(Debug)]
struct BasicCommand {
	/// The program to be executed. Panics if the argument does not expand to a single literal.
	program: Argument,
	/// Arguments to the program. The arguments may expand to an arbitrary number of literals.
	arguments: Box<[Argument]>,
	/// Redirections to be placed in order.
	redirections: Box<[Redirection]>,
	/// Whether to abort the command block execution if the command fails.
	abort_on_error: bool,
	pos: SourcePos,
}


impl BasicCommand {
	pub fn eval(self) -> Result<Child, Panic> {
		let pos = self.pos;

		let mut command = match self.program {
			Argument::Regex(_) => todo!(),
			Argument::Literal(program) => process::Command::new(program),
		};

		for argument in self.arguments.iter() {
			match argument {
				Argument::Regex(_) => todo!(),
				Argument::Literal(arg) => command.arg(arg),
			};
		}

		for redirection in self.redirections.iter() {
			todo!()
		}

		let child = command
			.spawn()
			.map_err(move |error| Panic::io(error, pos))?; // TODO: this should not be an error.

		Ok(child)
	}
}


/// Commands may be pipelines, or a single BasicCommand.
#[derive(Debug)]
enum Command {
	Builtin {
		/// The program to be executed.
		program: Builtin,
		/// Arguments to the program. The arguments may expand to an arbitrary number of literals.
		arguments: Box<[Argument]>,
		/// Whether to abort the command block execution if the command fails.
		abort_on_error: bool,
		pos: SourcePos,
	},
	External {
		/// The first command.
		head: BasicCommand,
		/// The following commands, if any.
		tail: Box<[BasicCommand]>
	}
}


impl Command {
	/// Returns a pair of result value and whether to abort on error.
	pub fn eval(self) -> Result<(Value, bool), Panic> {
		// todo: setup pipes
		todo!()
	}
}


/// A command block.
#[derive(Debug)]
struct CommandBlock {
	pub head: Command,
	pub tail: Box<[Command]>,
}


impl CommandBlock {
	pub fn eval(self) -> Result<Value, Panic> {
		// todo: return single if tail is empty, or an array otherwise.
		todo!()
	}
}


impl<'a> Runtime<'a> {
	pub(super) fn eval_command_block(
		&mut self,
		block: &'static program::CommandBlock
	) -> Result<Value, Panic> {
		match block.kind {
			program::CommandBlockKind::Synchronous => {
				let command_block = self.build_command_block(&block.head, &block.tail)?;
				command_block.eval()
			}

			program::CommandBlockKind::Asynchronous => todo!(),
			program::CommandBlockKind::Capture => todo!(),
		}
	}


	fn build_command_block(
		&mut self,
		head: &'static program::Command,
		tail: &'static [program::Command],
	) -> Result<CommandBlock, Panic> {
		let head = self.build_command(head)?;
		let tail = tail
			.iter()
			.map(
				|cmd| self.build_command(cmd)
			)
			.collect::<Result<_, Panic>>()?;

		Ok(CommandBlock { head, tail })
	}


	fn build_command(&mut self, command: &'static program::Command) -> Result<Command, Panic> {
		match command {
			program::Command::Builtin { program, arguments, abort_on_error, pos } => {
				let mut args = Vec::new();
				for argument in arguments.iter() {
					let arguments = self
						.build_argument(argument)?
						.into_vec();
					args.extend(arguments);
				}

				Ok(
					Command::Builtin {
						program: program.into(),
						arguments: args.into(),
						abort_on_error: *abort_on_error,
						pos: self.pos(*pos),
					}
				)
			}

			program::Command::External { head, tail } => {
				let head = self.build_basic_command(head)?;
				let tail = tail
					.iter()
					.map(
						|cmd| self.build_basic_command(cmd)
					)
					.collect::<Result<_, Panic>>()?;

				Ok(Command::External { head, tail })
			}
		}
	}


	fn build_basic_command(
		&mut self,
		command: &'static program::BasicCommand,
	) -> Result<BasicCommand, Panic> {
		let program_pos = self.pos(command.program.pos);

		let program = self.build_single_argument(
			&command.program,
			|items| Panic::invalid_command_args("program", items, program_pos)
		)?;

		let mut args = Vec::new();
		for argument in command.arguments.iter() {
			let arguments = self
				.build_argument(argument)?
				.into_vec();
			args.extend(arguments);
		}

		let redirections = command.redirections
			.iter()
			.map(
				|cmd| self.build_redirection(cmd)
			)
			.collect::<Result<_, Panic>>()?;

		Ok(
			BasicCommand {
				program,
				arguments: args.into(),
				redirections,
				abort_on_error: command.abort_on_error,
				pos: self.pos(command.pos),
			}
		)
	}


	fn build_redirection(
		&mut self,
		redirection: &'static program::Redirection,
	) -> Result<Redirection, Panic> {
		match redirection {
			program::Redirection::Output { source, target } => {
				let target = self.build_redirection_target(target)?;

				Ok(Redirection::Output { source: *source, target })
			}

			program::Redirection::Input { literal, source } => {
				let pos = self.pos(source.pos);

				let source = self.build_single_argument(
					source,
					|items| Panic::invalid_command_args("redirection", items, pos)
				)?;

				Ok(Redirection::Input { literal: *literal, source })
			}
		}
	}


	fn build_redirection_target(
		&mut self,
		target: &'static program::RedirectionTarget,
	) -> Result<RedirectionTarget, Panic> {
		match target {
			program::RedirectionTarget::Fd(fd) => Ok(RedirectionTarget::Fd(*fd)),

			program::RedirectionTarget::Overwrite(arg) => {
				let pos = self.pos(arg.pos);

				let target = self.build_single_argument(
					arg,
					|items| Panic::invalid_command_args("redirection", items, pos)
				)?;

				Ok(RedirectionTarget::Overwrite(target))
			}

			program::RedirectionTarget::Append(arg) => {
				let pos = self.pos(arg.pos);

				let target = self.build_single_argument(
					arg,
					|items| Panic::invalid_command_args("redirection", items, pos)
				)?;

				Ok(RedirectionTarget::Append(target))
			},
		}
	}


	fn build_single_argument<P>(
		&mut self,
		argument: &'static program::Argument,
		panic: P,
	) -> Result<Argument, Panic>
	where
		P: FnOnce(u32) -> Panic
	{
		let mut args = self
			.build_argument(argument)?
			.into_vec();

		let arg =
			if let Some(arg) = args.pop() {
				arg
			} else {
				return Err(panic(0))
			};

		if args.is_empty() {
			Ok(arg)
		} else {
			Err(panic(args.len() as u32))
		}
	}


	fn build_argument(
		&mut self,
		argument: &'static program::Argument,
	) -> Result<Box<[Argument]>, Panic> {
		let mut args = Args::default();

		for part in argument.parts.iter() {
			match part {
				program::ArgPart::Unit(program::ArgUnit::Literal(lit)) => {
					args.push_literal(lit);
				}

				program::ArgPart::Unit(program::ArgUnit::Dollar { slot_ix, pos }) => {
					let value = self.stack.fetch(slot_ix.into());

					match value {
						Value::Array(ref array) => {
							let literals: Vec<Cow<[u8]>> = array
								.borrow()
								.iter()
								.map(
									|val| {
										let lit = Self::build_basic_value(val.copy(), self.pos(*pos))?;
										Ok(Cow::Owned(lit.into_vec()))
									}
								)
								.collect::<Result<_, Panic>>()?;

							args.push_literals(literals.iter());

						}

						other => {
							let lit = Self::build_basic_value(other, self.pos(*pos))?;
							args.push_literal(&lit);
						}
					}
				}

				program::ArgPart::Home => {
					// TODO: is it possible to emit an error value here?
					let home = std::env::var_os("HOME").unwrap_or_default();

					args.push_literal(home.as_bytes());
				}

				program::ArgPart::Range(from, to) => {
					let items = (*from ..= *to)
						.map(
							|i| i.to_string().into_bytes()
						);

					args.push_literals(items);
				},

				program::ArgPart::Collection(items) => {
					let literals: Vec<Cow<[u8]>> = items
						.iter()
						.map(
							|unit| match unit {
								program::ArgUnit::Literal(lit) => Ok(Cow::Borrowed(lit.as_ref())),
								program::ArgUnit::Dollar { slot_ix, pos } => {
									let value = self.stack.fetch(slot_ix.into());
									let lit = Self::build_basic_value(value, self.pos(*pos))?;
									Ok(Cow::Owned(lit.into_vec()))
								},
							}
						)
						.collect::<Result<_, Panic>>()?;

					args.push_literals(literals.iter());
				},

				program::ArgPart::Star => args.push_regex(b".*"),
				program::ArgPart::Question => args.push_regex(b"?"),
				program::ArgPart::CharClass(class) => {
					args.push_regex(b"[");
					args.push_regex(class);
					args.push_regex(b"]");
				}
			}
		}

		Ok(args.into())
	}


	fn build_basic_value(value: Value, pos: SourcePos) -> Result<Box<[u8]>, Panic> {
		let string: Option<Vec<u8>> = match &value {
			Value::Nil => Some(b"nil".to_owned().to_vec()),
			Value::Bool(b) => Some(b.to_string().into()),
			Value::Int(int) => Some(int.to_string().into()),
			Value::Float(float) => Some(float.to_string().into()),
			Value::Byte(byte) => Some(vec![*byte]),
			Value::String(string) => Some(AsRef::<[u8]>::as_ref(string).to_owned()),

			Value::Array(_) => None,
			Value::Dict(_) => None,
			Value::Function(_) => None,
			Value::Error(_) => None,
		};

		string
			.map(Into::into)
			.ok_or(Panic::type_error(value, pos))
	}
}
