mod arg;
mod exec;

use std::{
	borrow::Cow,
	collections::HashMap,
	os::unix::{ffi::OsStrExt, prelude::OsStringExt},
	path::PathBuf,
	ops::DerefMut, io::Read, ffi::{OsStr, OsString}, thread
};

use super::{
	program,
	Dict,
	Panic,
	Runtime,
	SourcePos,
	Value,
};
use arg::Args;
use exec::IntoValue;


impl Runtime {
	pub(super) fn eval_command_block(
		&mut self,
		block: &'static program::CommandBlock,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let command_block = self.build_command_block(&block.head, &block.tail)?;

		match block.kind {
			program::CommandBlockKind::Synchronous => {
				command_block
					.exec(
						os_pipe::dup_stdout,
						os_pipe::dup_stderr,
					)
					.map(|errors| errors.into_value(self.interner()))
					.map_err(Into::into)
			}

			program::CommandBlockKind::Capture => {
				thread_local! {
					pub static ERROR: Value = "error".into();
					pub static STDOUT: Value = "stdout".into();
					pub static STDERR: Value = "stderr".into();
				}

				let (mut stdout_read, stdout_write) = os_pipe::pipe()
					.map_err(|error| Panic::io(error, pos.copy()))?;

				let (mut stderr_read, stderr_write) = os_pipe::pipe()
					.map_err(|error| Panic::io(error, pos.copy()))?;

				let stdout_reader = thread::spawn(move || {
					let mut data = Vec::with_capacity(512);
					stdout_read.read_to_end(&mut data)?;
					Ok(data)
				});

				let stderr_reader = thread::spawn(move || {
					let mut data = Vec::with_capacity(512);
					stderr_read.read_to_end(&mut data)?;
					Ok(data)
				});

				let errors = command_block
					.exec(
						// We must drop all writers before attempting to read, otherwise we'll deadlock.
						move || stdout_write.try_clone(),
						move || stderr_write.try_clone(),
					)
					.map_err(Panic::from)?;

				let mut result = errors.into_value(self.interner());
				let mut captures = {
					let out = match stdout_reader.join() {
						Err(error) => std::panic::resume_unwind(error),
						Ok(result) => result
							.map_err(|error| Panic::io(error, pos.copy()))?
							.into_boxed_slice(),
					};

					let err = match stderr_reader.join() {
						Err(error) => std::panic::resume_unwind(error),
						Ok(result) => result
							.map_err(|error| Panic::io(error, pos.copy()))?
							.into_boxed_slice(),
					};

					let mut dict = HashMap::new();

					STDOUT.with(
						|stdout| dict.insert(stdout.copy(), out.into())
					);
					STDERR.with(
						|stderr| dict.insert(stderr.copy(), err.into())
					);

					dict
				};

				match &mut result {
					Value::Nil => Ok(Dict::new(captures).into()),
					Value::Error(error) => {
						let ctx = std::mem::take(error.context.borrow_mut().deref_mut());

						ERROR.with(
							|error| captures.insert(error.copy(), ctx)
						);

						*error.context.borrow_mut() = Dict::new(captures).into();

						Ok(result)
					},
					_ => unreachable!("exec should only produce nil or error"),
				}
			}

			program::CommandBlockKind::Asynchronous => {
				thread_local! {
					pub static JOIN: Value = "join".into();
				}

				let join_handle = std::thread::spawn(
					|| command_block.exec(
						os_pipe::dup_stdout,
						os_pipe::dup_stderr,
					)
				);

				let join_handle = exec::Join
					::new(join_handle)
					.into();

				let mut dict = HashMap::new();

				JOIN.with(
					|join| dict.insert(join.copy(), join_handle)
				);

				Ok(Dict::new(dict).into())
			}
		}
	}


	fn build_command_block(
		&mut self,
		head: &'static program::Command,
		tail: &'static [program::Command],
	) -> Result<exec::Block, Panic> {
		let head = self.build_command(head)?;
		let tail = tail
			.iter()
			.map(
				|cmd| self.build_command(cmd)
			)
			.collect::<Result<_, Panic>>()?;

		Ok(exec::Block { head, tail })
	}


	fn build_command(
		&mut self,
		command: &'static program::Command
	) -> Result<exec::Command, Panic> {
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
					exec::Command::Builtin {
						program: program.into(),
						arguments: args.into(),
						abort_on_error: *abort_on_error,
						pos: pos.into(),
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

				Ok(exec::Command::External { head, tail })
			}
		}
	}


	fn build_basic_command(
		&mut self,
		command: &'static program::BasicCommand,
	) -> Result<exec::BasicCommand, Panic> {
		let program_pos = command.program.pos.into();

		let program = self.build_single_argument(
			&command.program,
			|items| Panic::invalid_command_args("program", items, program_pos)
		)?;

		let env = self.build_env_vars(&command.env)?;

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
			exec::BasicCommand {
				program,
				env,
				arguments: args.into(),
				redirections,
				abort_on_error: command.abort_on_error,
				pos: command.pos.into(),
			}
		)
	}


	#[allow(clippy::type_complexity)]
	fn build_env_vars(
		&mut self,
		input_env: &'static [(program::ArgUnit, program::Argument)],
	) -> Result<Box<[(Box<OsStr>, exec::Argument)]>, Panic> {
		let mut env = Vec::new();
		for (key, value) in input_env.iter() {
			let pos = value.pos;

			let key = match key {
				program::ArgUnit::Literal(lit) => lit.clone(),
				program::ArgUnit::Dollar { slot_ix, pos } => {
					let value = self.stack.fetch(slot_ix.into());
					let lit = Self::build_basic_value(value, pos.into())?;
					lit.clone()
				}
			};
			let key = OsString::from_vec(key.into()).into_boxed_os_str();

			let value = self.build_single_argument(
				value,
				|items| Panic::invalid_command_args("env", items, pos.into())
			)?;

			env.push((key, value))
		}

		Ok(env.into())
	}


	fn build_redirection(
		&mut self,
		redirection: &'static program::Redirection,
	) -> Result<exec::Redirection, Panic> {
		match redirection {
			program::Redirection::Output { source, target } => {
				let target = self.build_redirection_target(target)?;

				Ok(exec::Redirection::Output { source: *source, target })
			}

			program::Redirection::Input { literal, source } => {
				let pos = source.pos.into();

				let source = self.build_single_argument(
					source,
					|items| Panic::invalid_command_args("redirection", items, pos)
				)?;

				Ok(exec::Redirection::Input { literal: *literal, source })
			}
		}
	}


	fn build_redirection_target(
		&mut self,
		target: &'static program::RedirectionTarget,
	) -> Result<exec::RedirectionTarget, Panic> {
		match target {
			program::RedirectionTarget::Fd(fd) => Ok(exec::RedirectionTarget::Fd(*fd)),

			program::RedirectionTarget::Overwrite(arg) => {
				let pos = arg.pos.into();

				let target = self.build_single_argument(
					arg,
					|items| Panic::invalid_command_args("redirection", items, pos)
				)?;

				Ok(exec::RedirectionTarget::Overwrite(target))
			}

			program::RedirectionTarget::Append(arg) => {
				let pos = arg.pos.into();

				let target = self.build_single_argument(
					arg,
					|items| Panic::invalid_command_args("redirection", items, pos)
				)?;

				Ok(exec::RedirectionTarget::Append(target))
			},
		}
	}


	fn build_single_argument<P>(
		&mut self,
		argument: &'static program::Argument,
		panic: P,
	) -> Result<exec::Argument, Panic>
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
	) -> Result<Box<[exec::Argument]>, Panic> {
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
										let lit = Self::build_basic_value(val.copy(), pos.into())?;
										Ok(Cow::Owned(lit.into_vec()))
									}
								)
								.collect::<Result<_, Panic>>()?;

							args.push_literals(literals.iter());

						}

						other => {
							let lit = Self::build_basic_value(other, pos.into())?;
							args.push_literal(&lit);
						}
					}
				}

				program::ArgPart::Home => {
					// TODO: should we emit an error value here?
					let home = std::env::var_os("HOME")
						.map(
							|home| {
								let mut path = PathBuf::from(home);
								path.push("");
								path.into_os_string()
							}
						)
						.unwrap_or_default();

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
									let lit = Self::build_basic_value(value, pos.into())?;
									Ok(Cow::Owned(lit.into_vec()))
								},
							}
						)
						.collect::<Result<_, Panic>>()?;

					args.push_literals(literals.iter());
				},

				program::ArgPart::Star => args.push_pattern(b"*"),
				program::ArgPart::Percent => args.push_pattern(b"?"),
				program::ArgPart::CharClass(class) => {
					args.push_pattern(b"[");
					args.push_pattern(class);
					args.push_pattern(b"]");
				}
			}
		}

		Ok(args.into())
	}


	fn build_basic_value(value: Value, pos: SourcePos) -> Result<Box<[u8]>, Panic> {
		let literal: Option<Vec<u8>> = match &value {
			Value::Nil => Some(Vec::default()),
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

		literal
			.map(Into::into)
			.ok_or_else(|| Panic::type_error(value, "nil, bool, int, float, byte or string", pos))
	}
}
