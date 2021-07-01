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

use std::path::Path;

use term::color;

use args::{Args, Command};
use runtime::{Panic, SourcePos};


fn main() -> ! {
	let command = match args::parse(std::env::args_os()) {
		Ok(command) => command,
		Err(error) => {
			eprint!("{}", error);
			std::process::exit(1)
		}
	};

	let result = match command {
		Command::Run(args) => run(args),
		Command::Help(msg) | Command::Version(msg) => {
			println!("{}", msg);
			std::process::exit(0)
		},
	};


	let exit_code = match result {
		Ok(code) => code,
		Err(error) => {
			eprintln!("{}", error);
			1
		}
	};

	std::process::exit(exit_code)
}


fn run(args: Args) -> Result<i32, Panic> {
	let mut interner = symbol::Interner::new();
	let file_path = &*Box::leak(Path::new("<stdin>").into());

	let source = syntax::Source
		::from_reader(
			file_path,
			std::io::stdin().lock()
		)
    .map_err(|error| Panic::io(error, SourcePos::file(file_path)))?;

	// ----------------------------------------------------------------------------------------
	let syntactic_analysis = syntax::Analysis::analyze(source, &mut interner);

	for error in syntactic_analysis.errors.iter().take(20) {
		eprintln!(
			"{}: {}",
			color::Fg(color::Red, "Error"),
			fmt::Show(error, &interner)
		);
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
			for error in errors.into_iter().take(20) {
				eprintln!(
					"{}: {}",
					color::Fg(color::Red, "Error"),
					fmt::Show(error, &interner)
				);
			}

			return Ok(2);
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
	if !syntactic_analysis.errors.is_empty() {
		return Ok(2);
	}

	if args.check {
		return Ok(0)
	}

	let program = Box::leak(Box::new(program));

	runtime::Runtime::eval(program, &mut interner)?;

	Ok(0)
}
