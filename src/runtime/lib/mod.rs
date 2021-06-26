use std::{
	collections::HashMap,
	io::{self, Write}
};

use super::{
	Dict,
	Panic,
	RustFun,
	SourcePos,
	Value,
};


/// Instantiate the stdlib.
pub fn new() -> Value {
	let mut dict = HashMap::new();

	dict.insert(
		stringify!(print).into(),
		RustFun { name: "std.print", fun: print }.into()
	);
	dict.insert(
		stringify!(length).into(),
		RustFun { name: "std.length", fun: length }.into()
	);

	Dict::new(dict).into()
}


/// std.print
fn print(args: &mut [Value], pos: SourcePos) -> Result<Value, Panic> {
	let stdout = io::stdout();
	let mut stdout = stdout.lock();

	let mut iter = args.iter();

	if let Some(value) = iter.next() {
		write!(stdout, "{}", value)
			.map_err(|error| Panic::io(error, pos.clone()))?;
	}

	for value in iter {
		write!(stdout, "\t{}", value)
			.map_err(|error| Panic::io(error, pos.clone()))?;
	}

	writeln!(stdout)
		.map_err(|error| Panic::io(error, pos))?;

	Ok(Value::default())
}


/// std.length
fn length(args: &mut [Value], pos: SourcePos) -> Result<Value, Panic> {
	match &args {
		&[ Value::Array(ref array) ] => Ok(Value::Int(array.len())),
		&[ Value::Dict(ref dict) ] => Ok(Value::Int(dict.len())),
		&[ Value::String(ref string) ] => Ok(Value::Int(string.len() as i64)),
		&[ ref other ] => Err(Panic::type_error(other.copy(), pos)),
		_ => Err(Panic::invalid_args(args.len() as u32, 1, pos))
	}
}
