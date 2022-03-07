use std::{
	io,
	path::Path,
	os::unix::ffi::OsStrExt,
};

use crate::{fmt, semantic::ErrorsDisplayContext, symbol, syntax::{self, AnalysisDisplayContext}, tests};
use super::{program, Analyzer, Program, Errors};


fn test_dir<P, F>(path: P, mut check: F) -> io::Result<()>
where
	P: AsRef<Path>,
	F: FnMut(&Result<Program, Errors>) -> bool,
{
	let mut interner = symbol::Interner::new();

	tests::util::test_dir(
		path,
		move |path, file| {
			let path_symbol = interner.get_or_intern(path.as_os_str().as_bytes());
			let source = syntax::Source::from_reader(path_symbol, file)?;
			let syntactic_analysis = syntax::Analysis::analyze(&source, &mut interner);

			if !syntactic_analysis.errors.is_empty() {
				panic!(
					"{}",
					fmt::Show(
						syntactic_analysis,
						AnalysisDisplayContext {
							max_errors: None,
							interner: &interner,
						}
					)
				);
			}

			let result = Analyzer::analyze(syntactic_analysis.ast, &mut interner);

			if !check(&result) {
				match result {
					Ok(program) => panic!(
						"{}",
						fmt::Show(
							program,
							program::fmt::Context::from(&interner),
						)
					),

					Err(errors) => panic!(
						"{}",
						fmt::Show(
							errors,
							ErrorsDisplayContext {
								max_errors: None,
								interner: &interner,
							}
						)
					),
				}
			}

			Ok(())
		}
	)
}


#[test]
fn test_examples() -> io::Result<()> {
	test_dir(
		"examples/hush",
		Result::is_ok,
	)
}


#[test]
fn test_positive() -> io::Result<()> {
	test_dir(
		"src/semantic/tests/data/positive",
		Result::is_ok
	)
}


#[test]
fn test_negative() -> io::Result<()> {
	test_dir(
		"src/semantic/tests/data/negative",
		Result::is_err,
	)
}
