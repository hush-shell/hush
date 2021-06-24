#![allow(dead_code)] // This is temporarily used for the inital development.

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


fn main() -> std::io::Result<()> {
	let mut interner = symbol::Interner::new();
	let source = syntax::Source::from_reader(Path::new("<stdin>"), std::io::stdin().lock())?;

	runtime(source, &mut interner);

	Ok(())
}


fn lexer(source: syntax::Source, interner: &mut symbol::Interner) {
	use syntax::lexer::{Cursor, Lexer};

	let cursor = Cursor::from(source.contents.as_ref());
	let lexer = Lexer::new(cursor, interner);
	let tokens: Vec<_> = lexer.collect();

	for result in tokens {
		match result {
			Ok(token) => println!("{}", fmt::Show(token, &*interner)),
			Err(error) => {
				eprintln!("{}: {}", source.path.display(), error)
			}
		}
	}
}


fn syntax(source: syntax::Source, interner: &mut symbol::Interner) -> syntax::Analysis {
	let analysis = syntax::Analysis::analyze(source, interner);

	println!("{}", color::Fg(color::Yellow, "syntactic analysis"));

	for error in analysis.errors.iter().take(20) {
		eprintln!(
			"{}: {}",
			color::Fg(color::Red, "Error"),
			fmt::Show(error, &*interner)
		);
	}

	println!(
		"{}",
		fmt::Show(
			&analysis.ast,
			syntax::ast::fmt::Context::from(&*interner)
		)
	);

	analysis
}


fn semantic(
	source: syntax::Source,
	interner: &mut symbol::Interner
) -> Option<semantic::program::Program> {
	let syntactic_analysis = syntax(source, interner);

	println!("{}", color::Fg(color::Yellow, "semantic analysis"));

	match semantic::Analyzer::analyze(syntactic_analysis.ast, interner) {
		Err(errors) => {
			for error in errors.into_iter().take(20) {
				eprintln!(
					"{}: {}",
					color::Fg(color::Red, "Error"),
					fmt::Show(error, &*interner)
				);
			}

			None
		}

		Ok(program) => {
			println!(
				"{}",
				fmt::Show(
					&program,
					semantic::program::fmt::Context::from(&*interner)
				)
			);

			Some(program)
		},
	}
}


fn runtime(
	source: syntax::Source,
	interner: &mut symbol::Interner
) -> Option<runtime::value::Value> {
	if let Some(program) = semantic(source, interner) {
		println!("{}", color::Fg(color::Yellow, "runtime"));

		let program = Box::leak(Box::new(program));

		match runtime::Runtime::eval(program, interner) {
			Ok(value) => Some(value),
			Err(panic) => {
				eprintln!("{}", panic);
				None
			}
		}
	} else {
		None
	}
}
