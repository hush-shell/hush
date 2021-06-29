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
	dict.insert("len".into(), Length.into());
	dict.insert("iter".into(), Iter.into());
	dict.insert("type".into(), Type.into());
	dict.insert("push".into(), Push.into());
	dict.insert("is_empty".into(), IsEmpty.into());

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


/// std.type
#[derive(Trace, Finalize)]
struct Type;

impl NativeFun for Type {
	fn name(&self) -> &'static str { "std.type" }

	fn call(&mut self, args: &mut [Value], pos: SourcePos) -> Result<Value, Panic> {
		thread_local! {
			pub static NIL: Value = "nil".into();
			pub static BOOL: Value = "bool".into();
			pub static INT: Value = "int".into();
			pub static FLOAT: Value = "float".into();
			pub static BYTE: Value = "byte".into();
			pub static STRING: Value = "string".into();
			pub static ARRAY: Value = "array".into();
			pub static DICT: Value = "dict".into();
			pub static FUNCTION: Value = "function".into();
			pub static ERROR: Value = "error".into();
		}

		let typename = match args {
			[ Value::Nil ] => &NIL,
			[ Value::Bool(_) ] => &BOOL,
			[ Value::Int(_) ] => &INT,
			[ Value::Float(_) ] => &FLOAT,
			[ Value::Byte(_) ] => &BYTE,
			[ Value::String(_) ] => &STRING,
			[ Value::Array(_) ] => &ARRAY,
			[ Value::Dict(_) ] => &DICT,
			[ Value::Function(_) ] => &FUNCTION,
			[ Value::Error() ] => &ERROR,
			[] | [_, _, ..] => return Err(Panic::invalid_args(args.len() as u32, 1, pos)),
		};

		Ok(typename.with(Value::copy))
	}
}


/// std.length
#[derive(Trace, Finalize)]
struct Length;

impl NativeFun for Length {
	fn name(&self) -> &'static str { "std.len" }

	fn call(&mut self, args: &mut [Value], pos: SourcePos) -> Result<Value, Panic> {
		match args {
			[ Value::Array(ref array) ] => Ok(Value::Int(array.len())),
			[ Value::Dict(ref dict) ] => Ok(Value::Int(dict.len())),
			[ Value::String(ref string) ] => Ok(Value::Int(string.len() as i64)),
			[ ref other ] => Err(Panic::type_error(other.copy(), pos)),
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
		match args {
			[ Value::Array(ref array) ] => Ok(
				IterImpl::Array {
					array: array.copy(),
					ix: 0,
				}.into()
			),

			[ Value::Dict(ref dict) ] => Ok(
				IterImpl::Dict {
					entries: dict
						.borrow()
						.iter()
						.map(|(k, v)| (k.copy(), v.copy()))
						.collect()
				}.into()
			),

			[ Value::String(ref string) ] => Ok(
				IterImpl::String {
					string: string.copy(),
					ix: 0,
				}.into()
			),

			[ other ] => Err(Panic::type_error(other.copy(), pos)),
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

		keys::FINISHED.with(
			|finished| iteration.insert(finished.copy(), next.is_none().into())
		);

		if let Some(next) = next {
			keys::VALUE.with(
				|value| iteration.insert(value.copy(), next)
			);
		}

		Ok(Dict::new(iteration).into())
	}
}


/// std.push
#[derive(Trace, Finalize)]
struct Push;

impl NativeFun for Push {
	fn name(&self) -> &'static str { "std.push" }

	fn call(&mut self, args: &mut [Value], pos: SourcePos) -> Result<Value, Panic> {
		match args {
			[ Value::Array(ref mut array), value ] => {
				array.push(value.copy());
				Ok(Value::Nil)
			},

			// TODO: should this method work on strings? Remember, strings are immutable.
			// [ Value::String(ref mut string), value ] => {
			// 	Ok(Value::Nil)
			// },

			[ other, _ ] => Err(Panic::type_error(other.copy(), pos)),
			_ => Err(Panic::invalid_args(args.len() as u32, 1, pos))
		}
	}
}


/// std.is_empty
#[derive(Trace, Finalize)]
struct IsEmpty;

impl NativeFun for IsEmpty {
	fn name(&self) -> &'static str { "std.is_empty" }

	fn call(&mut self, args: &mut [Value], pos: SourcePos) -> Result<Value, Panic> {
		match args {
			[ Value::Array(ref array) ] => Ok(array.is_empty().into()),

			[ Value::Dict(ref dict) ] => Ok(dict.is_empty().into()),

			[ Value::String(ref string) ] => Ok(string.is_empty().into()),

			[ other ] => Err(Panic::type_error(other.copy(), pos)),
			_ => Err(Panic::invalid_args(args.len() as u32, 1, pos))
		}
	}
}
