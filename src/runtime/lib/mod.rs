use std::{
	collections::HashMap,
	io::{self, Write}
};

use super::{
	Dict,
	Panic,
	RustFun,
	Value,
};


/// Instantiate the stdlib.
pub fn new() -> Value {
	let mut dict = HashMap::new();

	let print = RustFun { name: "std.print", fun: print }.into();

	dict.insert(stringify!(print).into(), print);

	Dict::new(dict).into()
}


/// std.print
fn print(args: &mut [Value]) -> Result<Value, Panic> {
	let stdout = io::stdout();
	let mut stdout = stdout.lock();

	let mut iter = args.iter();

	if let Some(value) = iter.next() {
		write!(stdout, "{}", value)?;
	}

	for value in iter {
		write!(stdout, "\t{}", value)?;
	}

	writeln!(stdout)?;

	Ok(Value::default())
}
