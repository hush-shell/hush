mod arg;
mod exec;

use std::{
	borrow::Cow,
	collections::HashMap,
	os::unix::ffi::OsStrExt,
	process
};

use super::{
	program,
	Dict,
	Error,
	Panic,
	Runtime,
	SourcePos,
	Value,
};
use arg::Args;


impl Runtime {
	pub(super) fn eval_command_block(
		&mut self,
		block: &'static program::CommandBlock
	) -> Result<Value, Panic> {
		let command_block = self.build_command_block(&block.head, &block.tail)?;

		match block.kind {
			program::CommandBlockKind::Synchronous => command_block
				.exec(
					process::Stdio::inherit,
					process::Stdio::inherit,
				)
				.map(Into::into)
				.map_err(Into::into),

			program::CommandBlockKind::Capture => {
				thread_local! {
					pub static STATUS: Value = "status".into();
					pub static STDOUT: Value = "stdout".into();
					pub static STDERR: Value = "stderr".into();
				}

				let mut block_status = command_block
					.exec(
						process::Stdio::piped,
						process::Stdio::piped,
					)
					.map_err(Into::into)?;

				let out = std::mem::take(&mut block_status.stdout).into_boxed_slice();
				let err = std::mem::take(&mut block_status.stderr).into_boxed_slice();

				let mut dict = HashMap::new();

				STDOUT.with(
					|stdout| dict.insert(stdout.copy(), out.into())
				);
				STDERR.with(
					|stderr| dict.insert(stderr.copy(), err.into())
				);
				STATUS.with(
					|status| dict.insert(status.copy(), block_status.into())
				);

				Ok(Dict::new(dict).into())
			}

			program::CommandBlockKind::Asynchronous => {
				thread_local! {
					pub static JOIN: Value = "join".into();
				}

				let join_handle = std::thread::spawn(
					|| command_block.exec(
						process::Stdio::inherit,
						process::Stdio::inherit,
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
				arguments: args.into(),
				redirections,
				abort_on_error: command.abort_on_error,
				pos: command.pos.into(),
			}
		)
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
									let lit = Self::build_basic_value(value, pos.into())?;
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
		let literal: Option<Vec<u8>> = match &value {
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

		literal
			.map(Into::into)
			.ok_or_else(|| Panic::type_error(value, pos))
	}
}
