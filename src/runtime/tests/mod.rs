use std::{
	io,
	path::Path,
	os::unix::ffi::OsStrExt,
};

use crate::{fmt, symbol, syntax, semantic, tests};
use super::{Runtime, Value, Panic};


fn test_dir<P, F>(path: P, mut check: F) -> io::Result<()>
where
	P: AsRef<Path>,
	F: FnMut(&Result<Value, Panic>) -> bool,
{
	let mut interner = symbol::Interner::new();

	tests::util::test_dir(
		path,
		move |path, file| {
			let path_symbol = interner.get_or_intern(path.as_os_str().as_bytes());
			let source = syntax::Source::from_reader(path_symbol, file)?;
			let syntactic_analysis = syntax::Analysis::analyze(source, &mut interner);

			if !syntactic_analysis.errors.is_empty() {
				panic!("{}", fmt::Show(syntactic_analysis, &interner));
			}

			let semantic_analysis = semantic::Analyzer::analyze(syntactic_analysis.ast, &mut interner);
			let program = match semantic_analysis {
				Ok(program) => program,
				Err(errors) => panic!("{}", fmt::Show(errors, &interner)),
			};

			let program = Box::leak(Box::new(program));

			let result = Runtime::eval(program, &mut interner);

			if !check(&result) {
				match result {
					Ok(value) => panic!("{}", fmt::Show(value, &interner)),
					Err(panic) => panic!("{}", fmt::Show(panic, &interner)),
				}
			}

			Ok(())
		}
	)
}


#[test]
fn test_positive() -> io::Result<()> {
	test_dir(
		"src/runtime/tests/data/positive",
		Result::is_ok
	)
}


#[test]
fn test_negative() -> io::Result<()> {
	test_dir(
		"src/runtime/tests/data/negative",
		|result| matches!(result, Err(Panic::AssertionFailed { .. }))
	)
}
