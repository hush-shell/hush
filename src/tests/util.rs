use std::{
	io,
	fs::{self, File},
	path::{Path, PathBuf},
};


pub fn test_dir<P, F>(path: P, mut test: F) -> io::Result<()>
where
	P: AsRef<Path>,
	F: FnMut(&Path, File) -> io::Result<()>,
{
	let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	dir.push(path);

	fn run<F>(dir: &Path, test: &mut F) -> io::Result<()>
	where
		F: FnMut(&Path, File) -> io::Result<()>,
	{
		for entry in fs::read_dir(dir)? {
			let path = entry?.path();

			if path.is_dir() {
				run(&path, test)?;
			} else {
				let file = File::open(&path)?;
				test(&path, file)?;
			}
		}

		Ok(())
	}

	run(&dir, &mut test)
}
