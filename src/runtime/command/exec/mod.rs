mod error;
mod fmt;
mod join;

use std::{
	ffi::{OsStr, OsString},
	fs::{File, OpenOptions},
	io::{self, Write},
	os::unix::prelude::{FromRawFd, OsStrExt, ExitStatusExt, IntoRawFd},
	process,
};

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


/// An argument may expand to zero or more literals.
#[derive(Debug)]
pub enum Argument {
	/// A pattern to be matched to file names. May expand to zero or more literals.
	Pattern(Box<OsStr>),
	/// A single literal.
	Literal(Box<OsStr>),
}


impl Argument {
	/// Resolve the argument in the current directory.
	pub fn resolve(self, pos: SourcePos) -> Result<Box<[Box<OsStr>]>, Panic> {
		match self {
			Self::Literal(lit) => Ok(Box::new([lit])),
			Self::Pattern(pattern) => {
				let pattern = pattern.into_os_string();

				let pattern_str = pattern
					.into_string()
					.map_err(|pattern| Panic::invalid_pattern(pattern, pos.copy()))?;

				let is_absolute = pattern_str.starts_with('/');

				let entries = glob::glob(&pattern_str)
					.map_err(|_| Panic::invalid_pattern(pattern_str.into(), pos))?
					.filter_map(Result::ok)
					.map(
						|path| if is_absolute {
							OsString::from(path).into_boxed_os_str()
						} else {
							let mut new_path = OsString::with_capacity(2 + path.as_os_str().len());
							new_path.push("./");
							new_path.push(path);
							new_path.into_boxed_os_str()
						}
					)
					.collect();

				Ok(entries)
			},
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

				let args = arg.resolve(pos.copy())?;

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
	pub stdin: os_pipe::PipeReader,
	pub stdout: os_pipe::PipeWriter,
	pub stderr: os_pipe::PipeWriter,
}


/// A single command, including possible redirections and try operator.
#[derive(Debug)]
pub struct BasicCommand {
	/// The program to be executed. Panics if the argument does not expand to a single literal.
	pub program: Argument,
	/// Key-value pairs of environment variables.
	pub env: Box<[(Box<OsStr>, Argument)]>,
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

		let program_args = self.program.resolve(pos.copy())?;

		let mut command = match program_args.as_ref() {
			[ program ] => process::Command::new(program),
			other => return Err(
				Panic::invalid_args("program", other.len() as u32, pos.copy()).into()
			),
		};

		for (key, value) in self.env.into_vec() { // Use vec's owned iterator.
			let value = value.resolve(pos.copy())?;

			match value.as_ref() {
				[ value ] => command.env(key, value),
				other => return Err(
					Panic::invalid_args("env variable", other.len() as u32, pos.copy()).into()
				),
			};
		}

		for argument in self.arguments.into_vec() {
			let args = argument.resolve(pos.copy())?;

			for arg in args.iter() {
				command.arg(arg);
			}
		}

		Self::spawn(&mut command, stdio, self.redirections, self.pos)
	}


	fn spawn(
		command: &mut process::Command,
		mut stdio: Stdio,
		redirections: Box<[Redirection]>,
		pos: SourcePos,
	) -> Result<Child, Error> {
		for redirection in redirections.into_vec() { // Use vec's owned iterator.
			match redirection {
				Redirection::Output { source, target } => {
					let target = Self::resolve_target(target, &stdio, pos.copy())?;

					match source {
						1 => stdio.stdout = target,
						2 => stdio.stderr = target,
						other => return Err(
							Panic::unsupported_fd(other, pos.copy()).into()
						),
					};
				}

				Redirection::Input { literal, source } => {
					let args = source.resolve(pos.copy())?;

					let source = match args.as_ref() {
						[ source ] => source,
						other => return Err(
							Panic::invalid_args("redirection", other.len() as u32, pos.copy()).into()
						),
					};

					let stdin: os_pipe::PipeReader =
						if literal {
							let (reader, mut writer) = os_pipe::pipe()
								.map_err(|error| Error::io(error, pos.copy()))?;

							writer.write_all(source.as_bytes())
								.map_err(|error| Error::io(error, pos.copy()))?;

							writer.write_all(b"\n")
								.map_err(|error| Error::io(error, pos.copy()))?;

							reader
						} else {
							let file = File::open(source.as_ref())
								.map_err(|error| Error::io(error, pos.copy()))?
								.into_raw_fd();

							// SAFETY: converting from a FD originated from a File is fine.
							unsafe { os_pipe::PipeReader::from_raw_fd(file) }
						};

					stdio.stdin = stdin;
				}
			}
		}

		command.stdin(stdio.stdin);
		command.stdout(stdio.stdout);
		command.stderr(stdio.stderr);

		let process = command.spawn()
			.map_err(|error| Error::io(error, pos.copy()))?;

		Ok(Child { process, pos })
	}


	fn resolve_target(target: RedirectionTarget, stdio: &Stdio, pos: SourcePos) -> Result<os_pipe::PipeWriter, Error> {
		let open = |arg: Argument, append| {
			let args = arg.resolve(pos.copy())?;

			let file = match args.as_ref() {
				[ file ] => OpenOptions::new()
					.create(true)
					.write(true)
					.append(append)
					.truncate(!append)
					.open(file.as_ref())
					.map_err(|error| Error::io(error, pos.copy()))?
					.into_raw_fd(),

				other => return Err(
					Panic::invalid_args("redirection", other.len() as u32, pos.copy()).into()
				),
			};

			Ok(
				// SAFETY: converting from a FD originated from a File is fine.
				unsafe { os_pipe::PipeWriter::from_raw_fd(file) }
			)
		};

		match target {
			RedirectionTarget::Overwrite(arg) => open(arg, false),
			RedirectionTarget::Append(arg) => open(arg, true),
			RedirectionTarget::Fd(fd) => {
				let writer = match fd {
					1 => &stdio.stdout,
					2 => &stdio.stderr,
					other => return Err(
						Panic::unsupported_fd(other, pos.copy()).into()
					),
				};

				writer.try_clone()
					.map_err(|error| Error::io(error, pos))
			}
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
		stdout: os_pipe::PipeWriter,
		stderr: os_pipe::PipeWriter,
	) -> Result<CommandExec, Error> {
		match self {
			Command::Builtin { program, arguments, abort_on_error, pos } => {
				let error = program.exec(arguments, pos)?;
				let abort = abort_on_error && error.is_some();
				Ok(
					CommandExec {
						errors: error.into(),
						abort,
					}
				)
			}

			Command::External { head, tail } => {
				let mut last_stdout = stdout;
				let mut last_stderr = stderr;

				let mut tail_children = Vec::new();
				for cmd in tail.into_vec().into_iter().rev() {
					let child_abort_on_error = cmd.abort_on_error;

					let (pipe_reader, pipe_writer) = os_pipe::pipe()
						.map_err(|error| Error::io(error, cmd.pos.copy()))?;

					let child = cmd.exec(
						Stdio {
							stdin: pipe_reader,
							stdout: last_stdout,
							stderr: last_stderr,
						}
					)?;

					last_stdout = pipe_writer;
					last_stderr = os_pipe::dup_stderr()
						.map_err(|error| Error::io(error, child.pos.copy()))?;

					tail_children.push((child, child_abort_on_error));
				}

				let head_abort_on_error = head.abort_on_error;

				let stdin = os_pipe::dup_stdin()
					.map_err(|error| Error::io(error, head.pos.copy()))?;

				let head_child = head.exec(
					Stdio {
						stdin,
						stdout: last_stdout,
						stderr: last_stderr,
					}
				)?;

				let mut abort = false;
				let mut errors = Vec::new();

				// Wait on head command.
				if let Some(error) = ErrorStatus::wait_child(head_child) {
					abort |= head_abort_on_error;
					errors.push(error);
				}

				// Wait on tail commands.
				for (child, abort_on_error) in tail_children.into_iter().rev() {
					if let Some(error) = ErrorStatus::wait_child(child) {
						abort |= abort_on_error;
						errors.push(error);
					}
				}

				Ok(
					CommandExec {
						errors: errors.into(),
						abort,
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
	pub fn exec<F, G>(self, stdout: F, stderr: G) -> Result<Box<[PipelineErrors]>, Panic>
	where
		F: FnMut() -> io::Result<os_pipe::PipeWriter>,
		G: FnMut() -> io::Result<os_pipe::PipeWriter>,
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

				Ok(Box::new([PipelineErrors::from(error)]))
			},
		}
	}


	fn _exec<F, G>(self, mut stdout: F, mut stderr: G,) -> Result<Box<[PipelineErrors]>, Error>
	where
		F: FnMut() -> io::Result<os_pipe::PipeWriter>,
		G: FnMut() -> io::Result<os_pipe::PipeWriter>,
	{
		let mut errors = Vec::new();

		let pos = self.head.pos();
		let head = self.head.exec(
			stdout()
				.map_err(|error| Error::io(error, pos.copy()))?,
			stderr()
				.map_err(|error| Error::io(error, pos.copy()))?,
		)?;

		if !head.errors.is_empty() {
			errors.push(head.errors);
		}

		if head.abort {
			return Ok(errors.into())
		}

		for command in self.tail.into_vec() { // Use vec's owned iterator.
			let pos = command.pos();
			let child = command.exec(
				stdout()
					.map_err(|error| Error::io(error, pos.copy()))?,
				stderr()
					.map_err(|error| Error::io(error, pos.copy()))?,
			)?;

			if !child.errors.is_empty() {
				errors.push(child.errors);
			}

			if child.abort {
				break;
			}
		}

		Ok(errors.into())
	}
}
