use std::{
	collections::HashMap,
	io::{self, Write}
};

use super::{
	mem,
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
fn print(stack: &mut mem::Stack, args: mem::SlotIx) -> Result<Value, Panic> {
	let stdout = io::stdout();
	let mut stdout = stdout.lock();

	let mut iter = (0 .. args.0).into_iter().map(mem::SlotIx);

	if let Some(slot_ix) = iter.next() {
		let value = stack.fetch(slot_ix);
		write!(stdout, "{}", value)?;
	}

	for slot_ix in iter {
		let value = stack.fetch(slot_ix);
		write!(stdout, "\t{}", value)?;
	}

	writeln!(stdout)?;

	Ok(Value::default())
}
