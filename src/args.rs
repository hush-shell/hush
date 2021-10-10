use std::{ffi::{OsStr, OsString}, os::unix::ffi::OsStrExt, path::{Path, PathBuf}};

use clap::{AppSettings, clap_app, crate_authors, crate_description, crate_version};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command {
	Help(Box<str>),
	Version(Box<str>),
	Run(Args)
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Args {
	pub script_path: Option<PathBuf>,
	/// Check program with static analysis, but don't run.
	pub check: bool,
	/// Print the AST.
	pub print_ast: bool,
	/// Print the program.
	pub print_program: bool,
	/// Arguments for the script.
	pub script_args: Box<[Box<[u8]>]>
}


pub fn parse<A, T>(args: A) -> clap::Result<Command>
where
	A: IntoIterator<Item = T>,
	T: Into<OsString> + Clone
{
	let app =
		clap_app!(
			Hush =>
				(version: crate_version!())
				(author: crate_authors!())
				(about: crate_description!())
				(@arg check: --check "Perform only static analysis instead of executing.")
				(@arg ast: --ast "Print the AST")
				(@arg program: --program "Print the PROGAM")
				// The script path must not be a separate parameter because we must prevent clap
				// from parsing flags to the right of the script path.
				(@arg arguments: ... +allow_hyphen_values "Script and/or arguments")
		)
		.setting(AppSettings::TrailingVarArg);

	match app.get_matches_from_safe(args) {
		Ok(matches) => {
			let mut arguments = matches
				.values_of_os("arguments")
				.into_iter()
				.flatten()
				.map(OsStrExt::as_bytes);

			let mut script_args = Vec::new();
			let script_path = match arguments.next() {
				None => None,
				Some(b"-") => None,
				Some(arg) => {
					let path = Path::new(OsStr::from_bytes(arg));
					if path.is_file() {
						Some(path.to_owned())
					} else {
						script_args.push(arg.into());
						None
					}
				}
			};

			script_args.extend(arguments.map(Into::into));

			Ok(
				Command::Run(
					Args {
						script_path,
						check: matches.is_present("check"),
						print_ast: matches.is_present("ast"),
						print_program: matches.is_present("program"),
						script_args: script_args.into_boxed_slice(),
					}
				)
			)
		},

		Err(error) => match error.kind {
			clap::ErrorKind::HelpDisplayed => Ok(
				Command::Help(error.message.into_boxed_str())
			),
			clap::ErrorKind::VersionDisplayed => Ok(
				Command::Version(error.message.into_boxed_str())
			),
			_ => Err(error)
		}
	}
}
