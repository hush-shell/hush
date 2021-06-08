use std::{io, fs, path::{Path, PathBuf}};

use crate::{fmt, symbol};
use super::{Analysis, Source};


fn test_dir<P, F>(path: P, mut check: F) -> io::Result<()>
where
	P: AsRef<Path>,
	F: FnMut(&Analysis, &symbol::Interner),
{
	let mut examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	examples_dir.push(path);

	fn run<F>(dir: &Path, interner: &mut symbol::Interner, check: &mut F) -> io::Result<()>
	where
		F: FnMut(&Analysis, &symbol::Interner),
	{
		for entry in fs::read_dir(dir)? {
			let path = entry?.path();

			if path.is_dir() {
				run(&path, interner, check)?;
			} else {
				let source = Source::from_path(path)?;
				let analysis = Analysis::analyze(source, interner);

				check(&analysis, interner)
			}
		}

		Ok(())
	}

	let mut interner = symbol::Interner::new();

	run(&examples_dir, &mut interner, &mut check)
}


#[test]
fn test_examples() -> io::Result<()> {
	test_dir(
		"examples/hush",
		|analysis, interner| if !analysis.errors.is_empty() {
			panic!("{}", fmt::Show(analysis, interner));
		}
	)
}


#[test]
fn test_positive() -> io::Result<()> {
	test_dir(
		"src/syntax/tests/data/positive",
		|analysis, interner| if !analysis.errors.is_empty() {
			panic!("{}", fmt::Show(analysis, interner));
		}
	)
}


#[test]
fn test_negative() -> io::Result<()> {
	test_dir(
		"src/syntax/tests/data/negative",
		|analysis, interner| if analysis.errors.is_empty() {
			panic!("{}", fmt::Show(analysis, interner));
		}
	)
}
