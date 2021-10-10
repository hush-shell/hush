#![allow(dead_code)] // This is temporarily used for the inital development.

mod args;
mod fmt;
mod io;
mod runtime;
mod semantic;
mod symbol;
mod syntax;
mod term;
#[cfg(test)]
mod tests;

use std::os::unix::ffi::OsStrExt;

use term::color;

use args::{Args, Command};
use runtime::{Panic, SourcePos, Runtime};


#[derive(Debug)]
enum ExitStatus {
	Success,
	InvalidArgs,
	StaticError,
	Panic,
}


impl From<ExitStatus> for i32 {
	fn from(status: ExitStatus) -> Self {
		match status {
			ExitStatus::Success => 0,
			ExitStatus::InvalidArgs => 1,
			ExitStatus::StaticError => 2,
			ExitStatus::Panic => 127,
		}
	}
}


fn main() -> ! {
	let command = match args::parse(std::env::args_os()) {
		Ok(command) => command,
		Err(error) => {
			eprint!("{}", error);
			std::process::exit(ExitStatus::InvalidArgs.into())
		}
	};

	let exit_status = match command {
		Command::Run(args) => run(args),
		Command::Help(msg) | Command::Version(msg) => {
			println!("{}", msg);
			ExitStatus::Success
		},
	};

	std::process::exit(exit_status.into())
}


fn run(args: Args) -> ExitStatus {
	let mut interner = symbol::Interner::new();

	let (source, path) = match args.script_path {
		Some(path) => {
			let path = interner.get_or_intern(path.as_os_str().as_bytes());
			let source = syntax::Source::from_path(path, &mut interner);
			(source, path)
		},

		None => {
			let path = interner.get_or_intern("<stdin>");
			let source = syntax::Source::from_reader(path, std::io::stdin().lock());
			(source, path)
		},
	};

	let source = match source {
    Ok(source) => source,
    Err(error) => {
			eprintln!(
				"{}",
				fmt::Show(
					Panic::io(error, SourcePos::file(path)),
					&interner
				)
			);
			return ExitStatus::Panic;
		}
	};

	// ----------------------------------------------------------------------------------------
	let syntactic_analysis = syntax::Analysis::analyze(source, &mut interner);
	let has_syntax_errors = !syntactic_analysis.is_ok();

	if has_syntax_errors {
		eprint!("{}", fmt::Show(
			syntactic_analysis.errors,
			syntax::AnalysisDisplayContext {
				max_errors: Some(20),
				interner: &interner,
			}
		));
	}

	if args.print_ast {
		println!("{}", color::Fg(color::Yellow, "--------------------------------------------------"));
		println!(
			"{}",
			fmt::Show(
				&syntactic_analysis.ast,
				syntax::ast::fmt::Context::from(&interner)
			)
		);
		println!("{}", color::Fg(color::Yellow, "--------------------------------------------------"));
	}

	// ----------------------------------------------------------------------------------------
	let program = match semantic::Analyzer::analyze(syntactic_analysis.ast, &mut interner) {
		Ok(program) => program,
		Err(errors) => {
			eprint!("{}", fmt::Show(
				errors,
				semantic::ErrorsDisplayContext {
					max_errors: Some(20),
					interner: &interner,
				}
			));
			return ExitStatus::StaticError;
		}
	};

	if args.print_program {
		println!("{}", color::Fg(color::Yellow, "--------------------------------------------------"));
		println!(
			"{}",
			fmt::Show(
				&program,
				semantic::program::fmt::Context::from(&interner)
			)
		);
		println!("{}", color::Fg(color::Yellow, "--------------------------------------------------"));
	}

	// ----------------------------------------------------------------------------------------
	if has_syntax_errors {
		return ExitStatus::StaticError;
	}

	if args.check {
		return ExitStatus::Success;
	}

	let program = Box::leak(Box::new(program));
	let mut runtime = Runtime::new(
		args.script_args.into_vec(), // Use vec's owned iterator.
		interner
	);

	match runtime.eval(program) {
    Ok(_) => ExitStatus::Success,
    Err(panic) => {
			eprintln!("{}", fmt::Show(panic, runtime.interner()));
			ExitStatus::Panic
		}
	}
}
