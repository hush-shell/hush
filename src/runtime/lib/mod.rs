use std::{
	collections::HashMap,
	io::{self, Write},
};

use gc::{Finalize, Trace};

use super::{
	keys,
	Array,
	Dict,
	Panic,
	NativeFun,
	Str,
	SourcePos,
	Value,
};


/// Instantiate the stdlib.
pub fn new() -> Value {
	let mut dict = HashMap::new();

	dict.insert("print".into(), Print.into());
	dict.insert("length".into(), Length.into());
	dict.insert("iter".into(), Iter.into());

	Dict::new(dict).into()
}


/// std.print
#[derive(Trace, Finalize)]
struct Print;

impl NativeFun for Print {
	fn name(&self) -> &'static str { "std.print" }

	fn call(&mut self, args: &mut [Value], pos: SourcePos) -> Result<Value, Panic> {
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
}


/// std.length
#[derive(Trace, Finalize)]
struct Length;

impl NativeFun for Length {
	fn name(&self) -> &'static str { "std.length" }

	fn call(&mut self, args: &mut [Value], pos: SourcePos) -> Result<Value, Panic> {
		match &args {
			&[ Value::Array(ref array) ] => Ok(Value::Int(array.len())),
			&[ Value::Dict(ref dict) ] => Ok(Value::Int(dict.len())),
			&[ Value::String(ref string) ] => Ok(Value::Int(string.len() as i64)),
			&[ ref other ] => Err(Panic::type_error(other.copy(), pos)),
			_ => Err(Panic::invalid_args(args.len() as u32, 1, pos))
		}
	}
}


/// std.iter
#[derive(Trace, Finalize)]
struct Iter;

impl NativeFun for Iter {
	fn name(&self) -> &'static str { "std.iter" }

	fn call(&mut self, args: &mut [Value], pos: SourcePos) -> Result<Value, Panic> {
		match &args {
			&[ Value::Array(ref array) ] => Ok(
				IterImpl::Array {
					array: array.copy(),
					ix: 0,
				}.into()
			),

			&[ Value::Dict(ref dict) ] => Ok(
				IterImpl::Dict {
					entries: dict
						.borrow()
						.iter()
						.map(|(k, v)| (k.copy(), v.copy()))
						.collect()
				}.into()
			),

			&[ Value::String(ref string) ] => Ok(
				IterImpl::String {
					string: string.copy(),
					ix: 0,
				}.into()
			),

			&[ ref other ] => Err(Panic::type_error(other.copy(), pos)),
			_ => Err(Panic::invalid_args(args.len() as u32, 1, pos))
		}
	}
}


#[derive(Trace, Finalize)]
enum IterImpl {
	Array {
		array: Array,
		ix: i64,
	},
	String {
		string: Str,
		ix: i64,
	},
	Dict {
		entries: Vec<(Value, Value)>,
	}
}

impl NativeFun for IterImpl {
	fn name(&self) -> &'static str { "std.iter<impl>" }

	fn call(&mut self, args: &mut [Value], pos: SourcePos) -> Result<Value, Panic> {
		if !args.is_empty() {
			return Err(Panic::invalid_args(args.len() as u32, 0, pos));
		}

		let mut iteration = HashMap::new();

		let next = match self {
			IterImpl::Array { array, ix } => {
				if let Ok(value) = array.index(*ix) {
					*ix += 1;
					Some(value)
				} else {
					None
				}
			}

			IterImpl::String { string, ix } => {
				if let Ok(value) = string.index(*ix) {
					*ix += 1;
					Some(value)
				} else {
					None
				}
			}

			IterImpl::Dict { entries } => entries
				.pop()
				.map(
					|(k, v)| {
						let mut entry = HashMap::new();

						keys::KEY.with(
							|key| entry.insert(key.copy(), k)
						);

						keys::VALUE.with(
							|value| entry.insert(value.copy(), v)
						);

						Dict::new(entry).into()
					}
				)
		};

		if let Some(next) = next {
			keys::FINISHED.with(
				|finished| iteration.insert(finished.copy(), false.into())
			);

			keys::VALUE.with(
				|value| iteration.insert(value.copy(), next)
			);
		} else {
			keys::FINISHED.with(
				|finished| iteration.insert(finished.copy(), true.into())
			);
		}

		Ok(Dict::new(iteration).into())
	}
}
