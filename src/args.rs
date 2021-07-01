use std::ffi::OsString;

use clap::{clap_app, crate_authors, crate_version, crate_description};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command {
	Help(Box<str>),
	Version(Box<str>),
	Run(Args)
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Args {
	/// Check program with static analysis, but don't run.
	pub check: bool,
	/// Print the AST.
	pub print_ast: bool,
	/// Print the program.
	pub print_program: bool,
}


pub fn parse<A, T>(args: A) -> clap::Result<Command>
where
	A: IntoIterator<Item = T>,
	T: Into<OsString> + Clone
{
	let app = clap_app!(
		Hush =>
			(version: crate_version!())
			(author: crate_authors!())
			(about: crate_description!())
			(@arg check: --check "Perform only static analysis instead of executing.")
			(@arg ast: --ast "Print the AST")
			(@arg program: --program "Print the PROGAM")
	);

	match app.get_matches_from_safe(args) {
		Ok(matches) => Ok(
			Command::Run(
				Args {
					check: matches.is_present("check"),
					print_ast: matches.is_present("ast"),
					print_program: matches.is_present("program"),
				}
			)
		),

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
