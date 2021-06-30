use std::{
	fmt::Display,
	os::unix::ffi::OsStrExt,
};

use super::{
	Argument,
	RedirectionTarget,
	Redirection,
	Builtin,
	BasicCommand,
	Command,
	CommandBlock,
};

use crate::{
	syntax::lexer::CommandOperator,
	fmt::{self, Indentation},
	term::color,
};


impl Display for Argument {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		'"'.fmt(f)?;

		match self {
			Argument::Regex(regex) => regex.fmt(f)?,
			Argument::Literal(lit) => String::from_utf8_lossy(lit.as_bytes()).escape_debug().fmt(f)?,
		};

		'"'.fmt(f)
	}
}


impl Display for RedirectionTarget {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Fd(fd) => write!(f, ">{}", fd),

			Self::Overwrite(arg) => {
				">".fmt(f)?;
				arg.fmt(f)
			}

			Self::Append(arg) => {
				">>".fmt(f)?;
				arg.fmt(f)
			},
		}
	}
}


impl Display for Redirection {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Output { source, target } => {
				source.fmt(f)?;
				target.fmt(f)
			}

			Self::Input { literal: false, source } => {
				"<".fmt(f)?;
				source.fmt(f)
			}

			Self::Input { literal: true, source } => {
				"<<".fmt(f)?;
				source.fmt(f)
			}
		}
	}
}


impl Display for Builtin {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let command = match self {
			Builtin::Alias => "alias",
			Builtin::Cd => "cd",
		};

		color::Fg(color::Green, command).fmt(f)
	}
}


impl Display for BasicCommand {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.program.fmt(f)?;

		for arg in self.arguments.iter() {
			" ".fmt(f)?;
			arg.fmt(f)?;
		}

		for redirection in self.redirections.iter() {
			" ".fmt(f)?;
			redirection.fmt(f)?;
		}

		if self.abort_on_error {
			" ".fmt(f)?;
			CommandOperator::Try.fmt(f)?;
		}

		Ok(())
	}
}


impl Display for Command {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Command::Builtin { program, arguments, abort_on_error, .. } => {
				program.fmt(f)?;

				for arg in arguments.iter() {
					" ".fmt(f)?;
					arg.fmt(f)?;
				}

				if *abort_on_error {
					" ".fmt(f)?;
					CommandOperator::Try.fmt(f)?;
				}
			}

			Command::External { head, tail } => {
				head.fmt(f)?;

				for command in tail.iter() {
					"\n".fmt(f)?;
					Indentation(2).fmt(f)?;
					color::Fg(color::Yellow, "|").fmt(f)?;
					" ".fmt(f)?;
					command.fmt(f)?;
				}
			}
		}

		Ok(())
	}
}


impl Display for CommandBlock {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		"{\n".fmt(f)?;
		Indentation(1).fmt(f)?;

		fmt::sep_by(
			std::iter::once(&self.head).chain(self.tail.iter()),
			f,
			|cmd, f| {
				Indentation(1).fmt(f)?;
				cmd.fmt(f)
			},
			";\n",
		)?;

		"\n}".fmt(f)
	}
}
