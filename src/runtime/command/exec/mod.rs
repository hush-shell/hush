mod error;
mod fmt;
mod join;

use std::{
	ffi::{OsStr, OsString},
	fs::{File, OpenOptions},
	io::{self, Read, Write},
	os::unix::prelude::{FromRawFd, OsStrExt, ExitStatusExt},
	process,
};

use regex::bytes::Regex;

use crate::io::FileDescriptor;
use super::{program, SourcePos};
pub use join::Join;
pub use error::{Panic, Error, PipelineErrors, IntoValue};


/// Status to be produced when an IO error occurs
const IO_ERROR_STATUS: i32 = 0x7F;
/// Offset of a signal status, according to Bash and Dash.
const SIGNAL_STATUS_OFFSET: i32 = 0xFF;


/// Execution status of a single command.
#[derive(Debug)]
pub struct ErrorStatus {
	description: String,
	status: i32,
	pos: SourcePos,
}


impl ErrorStatus {
	/// Wait a child process, and return the status.
	fn wait_child(mut child: Child) -> Option<Self> {
		let status = match child.process.wait() {
			Ok(status) => status,
			Err(error) => return Some(
				Self {
					description: error.to_string(),
					status: IO_ERROR_STATUS,
					pos: child.pos,
				}
			)
		};

		let code = status
			.code()
			.or_else(
				|| status
					.signal()
					.map(
						|status| status + SIGNAL_STATUS_OFFSET
					)
			)
			.unwrap_or(255);

		if code == 0 {
			None
		} else {
			Some(
				Self {
					description: "command returned non-zero".into(),
					status: code,
					pos: child.pos,
				}
			)
		}
	}
}


/// Execution status of a command block.
#[derive(Debug)]
pub struct BlockStatus {
	pub errors: Box<[PipelineErrors]>,
	pub stdout: Vec<u8>,
	pub stderr: Vec<u8>,
}


impl From<PipelineErrors> for BlockStatus {
	fn from(error: PipelineErrors) -> Self {
		Self {
			errors: [error].into(),
			stdout: Vec::new(),
			stderr: Vec::new(),
		}
	}
}


/// An argument may expand to zero or more literals.
#[derive(Debug)]
pub enum Argument {
	/// A regex to be matched to file names. May expand to zero or more literals.
	Regex(Regex),
	/// A single literal.
	Literal(Box<OsStr>),
}


impl Argument {
	/// Resolve the argument in the current directory.
	pub fn resolve(self) -> io::Result<Box<[Box<OsStr>]>> {
		match self {
			Self::Regex(_) => todo!(),
			Self::Literal(lit) => Ok(Box::new([lit])),
		}
	}
}


/// The target of a redirection operation.
#[derive(Debug)]
pub enum RedirectionTarget {
	/// Redirect to a file descriptor.
	Fd(FileDescriptor),
	/// Overwrite a file. Panics if the argument does not expand to a single literal.
	Overwrite(Argument),
	/// Append to a file. Panics if the argument does not expand to a single literal.
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
		/// The source argument. Panics if the argument does not expand to a single literal.
		source: Argument,
	},
}


/// Built-in commands.
#[derive(Debug, Copy, Clone)]
pub enum Builtin {
	Alias,
	Cd,
}


impl Builtin {
	pub fn exec(
		self,
		arguments: Box<[Argument]>,
		pos: SourcePos,
	) -> Result<Option<ErrorStatus>, Error> {
		let mut arguments = arguments.into_vec();

		match self {
			Builtin::Alias => todo!(),

			Builtin::Cd => {
				let arg = arguments
					.pop()
					.ok_or_else(|| Panic::invalid_args("argument", 0, pos.copy()))?;

				if !arguments.is_empty() {
					return Err(
						Panic::invalid_args("argument", arguments.len() as u32 + 1, pos.copy()).into()
					);
				}

				let args = arg.resolve()
					.map_err(|error| Error::io(error, pos.copy()))?;

				match args.as_ref() {
					[ dir ] => std::env::set_current_dir(dir.as_ref())
						.map_err(|error| Error::io(error, pos.copy()))?,
					other => return Err(
						Panic::invalid_args("argument", other.len() as u32, pos).into()
					),
				};

				Ok(None)
			}
		}
	}
}


impl<'a> From<&'a program::command::Builtin> for Builtin {
	fn from(builtin: &'a program::command::Builtin) -> Self {
		match builtin {
			program::command::Builtin::Alias => Self::Alias,
			program::command::Builtin::Cd => Self::Cd,
		}
	}
}


#[derive(Debug)]
pub struct Stdio {
	pub stdin: process::Stdio,
	pub stdout: process::Stdio,
	pub stderr: process::Stdio,
}


/// A single command, including possible redirections and try operator.
#[derive(Debug)]
pub struct BasicCommand {
	/// The program to be executed. Panics if the argument does not expand to a single literal.
	pub program: Argument,
	/// Arguments to the program. The arguments may expand to an arbitrary number of literals.
	pub arguments: Box<[Argument]>,
	/// Redirections to be placed in order.
	pub redirections: Box<[Redirection]>,
	/// Whether to abort the command block execution if the command fails.
	pub abort_on_error: bool,
	/// Source position of the command.
	pub pos: SourcePos,
}


impl BasicCommand {
	pub fn exec(self, stdio: Stdio) -> Result<Child, Error> {
		let pos = self.pos.copy();

		let program_args = self.program.resolve()
			.map_err(|error| Error::io(error, pos.copy()))?;

		let mut command = match program_args.as_ref() {
			[ program ] => process::Command::new(program),
			other => return Err(
				Panic::invalid_args("program", other.len() as u32, pos.copy()).into()
			),
		};

		for argument in self.arguments.into_vec() {
			let args = argument.resolve()
				.map_err(|error| Error::io(error, pos.copy()))?;

			for arg in args.iter() {
				command.arg(arg);
			}
		}

		// Setup pipes before redirections, as in Bash.
		command.stdin(stdio.stdin);
		command.stdout(stdio.stdout);
		command.stderr(stdio.stderr);

		Self::spawn(&mut command, self.redirections, self.pos)
	}


	fn spawn(
		command: &mut process::Command,
		redirections: Box<[Redirection]>,
		pos: SourcePos,
	) -> Result<Child, Error> {
		let mut input = None;

		for redirection in redirections.into_vec() { // Use vec's owned iterator.
			match redirection {
				Redirection::Output { source, target } => {
					let target = Self::resolve_target(target, pos.copy())?;

					match source {
						1 => command.stdout(target),
						2 => command.stderr(target),
						other => return Err(
							Panic::unsupported_fd(other, pos.copy()).into()
						),
					};
				}

				Redirection::Input { literal, source } => {
					let args = source.resolve()
						.map_err(|error| Error::io(error, pos.copy()))?;

					let source = match args.as_ref() {
						[ source ] => source,
						other => return Err(
							Panic::invalid_args("redirection", other.len() as u32, pos.copy()).into()
						),
					};

					let stdin =
						if literal {
							input = Some(OsString::from(source));
							process::Stdio::piped()
						} else {
							let file = File::open(source.as_ref())
								.map_err(|error| Error::io(error, pos.copy()))?;
							process::Stdio::from(file)
						};

					command.stdin(stdin);
				}
			}
		}

		let mut process = command.spawn()
			.map_err(|error| Error::io(error, pos.copy()))?;

		if let Some(mut input) = input {
			input.push("\n");

			let mut stdin = process.stdin
				.take()
				.ok_or_else(|| Error::pipe_fail(pos.copy()))?;

			stdin.write_all(input.as_bytes())
				.map_err(|error| Error::io(error, pos.copy()))?;
		}

		Ok(Child { process, pos })
	}


	fn resolve_target(target: RedirectionTarget, pos: SourcePos) -> Result<process::Stdio, Error> {
		let open = |arg: Argument, append| {
			let args = arg.resolve()
				.map_err(|error| Error::io(error, pos.copy()))?;

			let file = match args.as_ref() {
				[ file ] => OpenOptions::new()
					.create(true)
					.write(true)
					.append(append)
					.open(file.as_ref())
					.map_err(|error| Error::io(error, pos.copy()))?,

				other => return Err(
					Panic::invalid_args("redirection", other.len() as u32, pos).into()
				),
			};

			Ok(process::Stdio::from(file))
		};

		match target {
			RedirectionTarget::Fd(fd) => Ok(
				unsafe { process::Stdio::from_raw_fd(fd) } // TODO: test
			),
			RedirectionTarget::Overwrite(arg) => open(arg, false),
			RedirectionTarget::Append(arg) => open(arg, true),
		}
	}
}


#[derive(Debug)]
pub struct Child {
	process: process::Child,
	pos: SourcePos,
}


#[derive(Debug)]
pub struct CommandExec {
	pub errors: PipelineErrors,
	pub abort: bool,
	pub stdout: Option<process::ChildStdout>,
	pub stderr: Option<process::ChildStderr>,
}


/// Commands may be pipelines, or a single BasicCommand.
#[derive(Debug)]
pub enum Command {
	Builtin {
		/// The program to be executed.
		program: Builtin,
		/// Arguments to the program. The arguments may expand to an arbitrary number of literals.
		arguments: Box<[Argument]>,
		/// Whether to abort the command block execution if the command fails.
		abort_on_error: bool,
		/// Source position of the command.
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
	/// Returns a pair of result value and whether to abort.
	pub fn exec(
		self,
		stdout: process::Stdio,
		stderr: process::Stdio,
	) -> Result<CommandExec, Error> {
		match self {
			Command::Builtin { program, arguments, abort_on_error, pos } => {
				let error = program.exec(arguments, pos)?;
				let abort = abort_on_error && error.is_some();
				Ok(
					CommandExec {
						errors: error.into(),
						abort,
						stdout: None,
						stderr: None,
					}
				)
			}

			Command::External { head, tail } => {
				let mut last_stdout = stdout;
				let mut last_stderr = stderr;

				let mut tail_children = Vec::new();
				for cmd in tail.into_vec().into_iter().rev() {
					let child_abort_on_error = cmd.abort_on_error;
					let mut child = cmd.exec(
						Stdio {
							stdin: process::Stdio::piped(),
							stdout: last_stdout,
							stderr: last_stderr,
						}
					)?;

					last_stdout = child.process.stdin
						.take()
						.ok_or_else(|| Error::pipe_fail(child.pos.copy()))?
						.into();
					last_stderr = process::Stdio::inherit();

					tail_children.push((child, child_abort_on_error));
				}

				let head_abort_on_error = head.abort_on_error;
				let mut head_child = head.exec(
					Stdio {
						stdin: process::Stdio::inherit(),
						stdout: last_stdout,
						stderr: last_stderr,
					}
				)?;

				let mut abort = false;
				let mut errors = Vec::new();
				let mut last_stdout = head_child.process.stdout.take();
				let mut last_stderr = head_child.process.stderr.take();

				// Wait on head command.
				if let Some(error) = ErrorStatus::wait_child(head_child) {
					abort |= head_abort_on_error;
					errors.push(error.into());
				}

				// Wait on tail commands.
				for (mut child, abort_on_error) in tail_children.into_iter().rev() {
					last_stdout = child.process.stdout.take();
					last_stderr = child.process.stderr.take();
					if let Some(error) = ErrorStatus::wait_child(child) {
						abort |= abort_on_error;
						errors.push(error);
					}
				}

				Ok(
					CommandExec {
						errors: errors.into(),
						abort,
						stdout: last_stdout,
						stderr: last_stderr,
					}
				)
			}
		}
	}

	pub fn pos(&self) -> SourcePos {
		match self {
			Command::Builtin { pos, .. } => pos.copy(),
			Command::External { head, .. } => head.pos.copy(),
		}
	}
}


/// A command block.
#[derive(Debug)]
pub struct Block {
	pub head: Command,
	pub tail: Box<[Command]>,
}


impl Block {
	pub fn exec<F, G>(self, stdout: F, stderr: G) -> Result<BlockStatus, Panic>
	where
		F: FnMut() -> process::Stdio,
		G: FnMut() -> process::Stdio,
	{
		match self._exec(stdout, stderr) {
			Ok(status) => Ok(status),
			Err(Error::Panic(panic)) => Err(panic),
			Err(Error::Io { error, pos }) => {
				let error = ErrorStatus {
					description: error.to_string(),
					status: IO_ERROR_STATUS,
					pos,
				};

				Ok(PipelineErrors::from(error).into())
			},
		}
	}


	fn _exec<F, G>(self, mut stdout: F, mut stderr: G,) -> Result<BlockStatus, Error>
	where
		F: FnMut() -> process::Stdio,
		G: FnMut() -> process::Stdio,
	{
		let mut errors = Vec::new();
		let mut stdout_data = Vec::new();
		let mut stderr_data = Vec::new();

		let pos = self.head.pos();
		let head = self.head.exec(stdout(), stderr())?;

		if !head.errors.is_empty() {
			errors.push(head.errors);
		}
		if let Some(mut data) = head.stdout {
			data.read_to_end(&mut stdout_data)
				.map_err(|error| Error::io(error, pos.copy()))?;
		}
		if let Some(mut data) = head.stderr {
			data.read_to_end(&mut stderr_data)
				.map_err(|error| Error::io(error, pos))?;
		}

		if head.abort {
			return Ok(
				BlockStatus {
					errors: errors.into(),
					stdout: stdout_data,
					stderr: stderr_data,
				}
			)
		}

		for command in self.tail.into_vec() { // Use vec's owned iterator.
			let pos = command.pos();
			let child = command.exec(stdout(), stderr())?;

			if !child.errors.is_empty() {
				errors.push(child.errors);
			}
			if let Some(mut data) = child.stdout {
				data.read_to_end(&mut stdout_data)
					.map_err(|error| Error::io(error, pos.copy()))?;
			}
			if let Some(mut data) = child.stderr {
				data.read_to_end(&mut stderr_data)
					.map_err(|error| Error::io(error, pos))?;
			}

			if child.abort {
				break;
			}
		}

		Ok(
			BlockStatus {
				errors: errors.into(),
				stdout: stdout_data,
				stderr: stderr_data,
			}
		)
	}
}
