#[allow(dead_code)] // This is temporarily used for the inital development.
mod lexer;
mod symbol;
mod source;

use lexer::{Lexer, cursor::Cursor};

use std::io::{self, Read};


fn main() -> io::Result<()> {
	let mut input = Vec::with_capacity(512);
	io::stdin().lock().read_to_end(&mut input)?;

	let cursor = Cursor::new(&input);
	let mut interner = symbol::Interner::new();
	let mut lexer = Lexer::new(cursor, &mut interner);

	let mut previous_line = 0;
	while let Some(output) = lexer.next() {
		match output {
			Ok(token) => {
				if token.pos.line != previous_line {
					previous_line = token.pos.line;
					println!();
				}

				print!("{:?} ", token);
			}
			Err(error) => {
				if error.pos.line != previous_line {
					previous_line = error.pos.line;
				}

				eprintln!("\nerror: {}", error)
			}
		}
	}

	Ok(())
}
