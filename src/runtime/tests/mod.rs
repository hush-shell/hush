use std::{
	io,
	path::Path,
	os::unix::ffi::OsStrExt,
};

use serial_test::serial;

use crate::{
	fmt,
	semantic::{self, ErrorsDisplayContext},
	symbol,
	syntax::{self, AnalysisDisplayContext},
	tests,
};
use super::{Runtime, Value, Panic};


fn test_dir<P, F>(path: P, mut check: F) -> io::Result<()>
where
	P: AsRef<Path>,
	F: FnMut(&Result<Value, Panic>) -> bool,
{
	let interner = symbol::Interner::new();
	let args = std::iter::empty::<&str>();
	let mut runtime = Runtime::new(args, interner);

	tests::util::test_dir(
		path,
		move |path, file| {
			let path_symbol = runtime
				.interner_mut()
				.get_or_intern(path.as_os_str().as_bytes());
			let source = syntax::Source::from_reader(path_symbol, file)?;
			let syntactic_analysis = syntax::Analysis::analyze(
				&source,
				runtime.interner_mut()
			);

			if !syntactic_analysis.errors.is_empty() {
				panic!(
					"{}",
					fmt::Show(
						syntactic_analysis,
						AnalysisDisplayContext {
							max_errors: None,
							interner: runtime.interner(),
						}
					)
				);
			}

			let semantic_analysis = semantic::Analyzer::analyze(
				syntactic_analysis.ast,
				runtime.interner_mut()
			);
			let program = match semantic_analysis {
				Ok(program) => program,
				Err(errors) => panic!(
					"{}",
					fmt::Show(
						errors,
						ErrorsDisplayContext {
							max_errors: None,
							interner: runtime.interner(),
						}
					)
				),
			};

			let program = Box::leak(Box::new(program));

			let result = runtime.eval(program);

			if !check(&result) {
				match result {
					Ok(value) => panic!(
						"File {}: expected panic, got {}",
						path.display(),
						fmt::Show(value, runtime.interner())
					),
					Err(panic) => panic!("{}", fmt::Show(panic, runtime.interner())),
				}
			}

			Ok(())
		}
	)
}


// As our garbage collector is not thread safe, we must *not* run the following tests in
// parallel.


#[test]
#[serial]
fn test_positive() -> io::Result<()> {
	test_dir(
		"src/runtime/tests/data/positive",
		Result::is_ok
	)
}


#[test]
#[serial]
fn test_negative() -> io::Result<()> {
	test_dir(
		"src/runtime/tests/data/negative",
		Result::is_err
	)
}


#[test]
#[serial]
fn test_asserts() -> io::Result<()> {
	test_dir(
		"src/runtime/tests/data/negative/asserts",
		|result| matches!(result, Err(Panic::AssertionFailed { .. }))
	)
}
