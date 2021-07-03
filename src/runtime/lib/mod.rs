mod util;

use std::{
	collections::HashMap,
	ffi::OsStr,
	io::{self, Write},
	path::PathBuf,
};

use gc::{Finalize, Trace};

use crate::{
	fmt::{self, FmtString},
	symbol
};
use super::{
	keys,
	Array,
	Dict,
	Error,
	Float,
	Function,
	NativeFun,
	Panic,
	Runtime,
	SourcePos,
	Str,
	Value,
};


/// Instantiate the stdlib.
pub fn new() -> Value {
	let mut dict = HashMap::new();

	dict.insert("assert".into(), Assert.into());
	dict.insert("bind".into(), Bind.into());
	dict.insert("cd".into(), Cd.into());
	dict.insert("cwd".into(), Cwd.into());
	dict.insert("env".into(), Env.into());
	dict.insert("error".into(), ErrorFun.into());
	dict.insert("has_error".into(), HasError.into());
	dict.insert("is_empty".into(), IsEmpty.into());
	dict.insert("iter".into(), Iter.into());
	dict.insert("len".into(), Length.into());
	dict.insert("pop".into(), Pop.into());
	dict.insert("print".into(), Print.into());
	dict.insert("push".into(), Push.into());
	dict.insert("range".into(), Range.into());
	dict.insert("to_string".into(), ToString.into());
	dict.insert("type".into(), Type.into());

	Dict::new(dict).into()
}


/// std.print
#[derive(Trace, Finalize)]
struct Print;


impl Print {
	fn print<W: Write>(value: &Value, interner: &symbol::Interner, mut writer: W) -> io::Result<()> {
		match value {
			Value::String(string) => writer.write_all(string.as_ref()),
			Value::Byte(byte) => writer.write_all(&[*byte]),
			value => write!(writer, "{}", fmt::Show(value, interner)),
		}
	}
}


impl NativeFun for Print {
	fn name(&self) -> &'static str { "std.print" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

		let stdout = io::stdout();
		let mut stdout = stdout.lock();

		let mut iter = args.iter();

		if let Some(value) = iter.next() {
			Self::print(value, &runtime.interner, &mut stdout)
				.map_err(|error| Panic::io(error, pos.copy()))?;
		}

		for value in iter {
			write!(stdout, "\t")
				.map_err(|error| Panic::io(error, pos.copy()))?;

			Self::print(value, &runtime.interner, &mut stdout)
				.map_err(|error| Panic::io(error, pos.copy()))?;
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

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

		thread_local! {
			pub static NIL: Value = "nil".into();
			pub static BOOL: Value = "bool".into();
			pub static INT: Value = "int".into();
			pub static FLOAT: Value = "float".into();
			pub static BYTE: Value = "char".into();
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
			[ Value::Error(_) ] => &ERROR,
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

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

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

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

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

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

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

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &mut runtime.arguments[args_start..];

		match args {
			[ Value::Array(ref mut array), value ] => {
				array.push(value.copy());
				Ok(Value::Nil)
			},

			[ other, _ ] => Err(Panic::type_error(other.copy(), pos)),
			_ => Err(Panic::invalid_args(args.len() as u32, 2, pos))
		}
	}
}


/// std.pop
#[derive(Trace, Finalize)]
struct Pop;

impl NativeFun for Pop {
	fn name(&self) -> &'static str { "std.pop" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &mut runtime.arguments[args_start..];

		match args {
			[ Value::Array(ref mut array) ] => {
				let value = array
					.pop()
					.map_err(|_| Panic::empty_collection(pos))?;

				Ok(value)
			},

			[ other ] => Err(Panic::type_error(other.copy(), pos)),
			_ => Err(Panic::invalid_args(args.len() as u32, 1, pos))
		}
	}
}


/// std.is_empty
#[derive(Trace, Finalize)]
struct IsEmpty;

impl NativeFun for IsEmpty {
	fn name(&self) -> &'static str { "std.is_empty" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

		match args {
			[ Value::Array(ref array) ] => Ok(array.is_empty().into()),

			[ Value::Dict(ref dict) ] => Ok(dict.is_empty().into()),

			[ Value::String(ref string) ] => Ok(string.is_empty().into()),

			[ other ] => Err(Panic::type_error(other.copy(), pos)),
			_ => Err(Panic::invalid_args(args.len() as u32, 1, pos))
		}
	}
}


/// std.error
#[derive(Trace, Finalize)]
struct ErrorFun;

impl NativeFun for ErrorFun {
	fn name(&self) -> &'static str { "std.error" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

		match args {
			[ Value::String(ref string), context ] => Ok(
				Error
					::new(string.copy(), context.copy())
					.into()
			),

			[ other, _ ] => Err(Panic::type_error(other.copy(), pos)),
			_ => Err(Panic::invalid_args(args.len() as u32, 2, pos))
		}
	}
}


/// std.range
#[derive(Trace, Finalize)]
struct Range;

impl NativeFun for Range {
	fn name(&self) -> &'static str { "std.range" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

		match args {
			[ from, to, step ] => {
				let numbers = util::Numbers
					::promote([from.copy(), to.copy(), step.copy()])
					.map_err(|value| Panic::type_error(value, pos))?;

				Ok(
					match numbers {
						util::Numbers::Ints([ from, to, step ]) => RangeImpl { from, to, step }.into(),
						util::Numbers::Floats([ from, to, step ]) => RangeImpl { from, to, step }.into(),
					}
				)
			},

			[ other ] => Err(Panic::type_error(other.copy(), pos)),
			_ => Err(Panic::invalid_args(args.len() as u32, 3, pos))
		}
	}
}


#[derive(Trace, Finalize)]
struct RangeImpl<T> {
	from: T,
	to: T,
	step: T,
}

impl<T> NativeFun for RangeImpl<T>
where
	T: Trace + Finalize + 'static,
	T: Clone + Default + Ord + std::ops::Add<Output = T>,
	T: Into<Value>,
{
	fn name(&self) -> &'static str { "std.range<impl>" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

		if !args.is_empty() {
			return Err(Panic::invalid_args(args.len() as u32, 0, pos));
		}

		let mut iteration = HashMap::new();

		let finished =
			if self.step > T::default() { // Step is positive.
				self.from >= self.to
			} else { // Step is negative.
				self.from <= self.to
			};

		let next = if finished {
			None
		} else {
			let value = self.from.clone();
			self.from = self.from.clone() + self.step.clone();
			Some(value)
		};

		keys::FINISHED.with(
			|finished| iteration.insert(finished.copy(), next.is_none().into())
		);

		if let Some(next) = next {
			keys::VALUE.with(
				|value| iteration.insert(value.copy(), next.into())
			);
		}

		Ok(Dict::new(iteration).into())
	}
}


/// std.env
#[derive(Trace, Finalize)]
struct Env;

impl NativeFun for Env {
	fn name(&self) -> &'static str { "std.env" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

		match args {
			[ Value::String(ref string) ] => Ok(
				std::env
					::var_os(string)
					.map(Value::from)
					.unwrap_or(Value::Nil)
			),

			[ other ] => Err(Panic::type_error(other.copy(), pos)),
			_ => Err(Panic::invalid_args(args.len() as u32, 1, pos))
		}
	}
}


/// std.has_error
#[derive(Trace, Finalize)]
struct HasError;

impl HasError {
	fn has_error(value: &Value) -> bool {
		match value {
			Value::Error(_) => true,

			Value::Array(array) => {
				for value in array.borrow().iter() {
					if Self::has_error(value) {
						return true;
					}
				}

				false
			}

			Value::Dict(dict) => {
				for (key, value) in dict.borrow().iter() {
					if Self::has_error(key) || Self::has_error(value) {
						return true;
					}
				}

				false
			}

			_ => false,
		}
	}
}


impl NativeFun for HasError {
	fn name(&self) -> &'static str { "std.has_error" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

		match args {
			[ value ] => Ok(Self::has_error(value).into()),
			_ => Err(Panic::invalid_args(args.len() as u32, 1, pos))
		}
	}
}


/// std.to_string
#[derive(Trace, Finalize)]
struct ToString;

impl NativeFun for ToString {
	fn name(&self) -> &'static str { "std.to_string" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

		match args {
			[ Value::String(ref string) ] => Ok(string.copy().into()),
			[ value ] => Ok(value.fmt_string(&runtime.interner).into()),
			_ => Err(Panic::invalid_args(args.len() as u32, 1, pos))
		}
	}
}


/// std.cd
#[derive(Trace, Finalize)]
struct Cd;

impl NativeFun for Cd {
	fn name(&self) -> &'static str { "std.cd" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

		match args {
			[ Value::String(ref string) ] => Ok(
				std::env
					::set_current_dir(AsRef::<OsStr>::as_ref(string))
					.into()
			),

			[ other ] => Err(Panic::type_error(other.copy(), pos)),
			_ => Err(Panic::invalid_args(args.len() as u32, 1, pos))
		}
	}
}


/// std.cwd
#[derive(Trace, Finalize)]
struct Cwd;

impl NativeFun for Cwd {
	fn name(&self) -> &'static str { "std.cwd" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

		if !args.is_empty() {
			return Err(Panic::invalid_args(args.len() as u32, 0, pos));
		}

		Ok(
			std::env
				::current_dir()
				.map(PathBuf::into_os_string)
				.into()
		)
	}
}


/// std.assert
#[derive(Trace, Finalize)]
struct Assert;

impl NativeFun for Assert {
	fn name(&self) -> &'static str { "std.assert" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

		match args {
			[ Value::Bool(true) ] => Ok(Value::default()),
			[ Value::Bool(false) ] => Err(Panic::assertion_failed(pos)),

			[ other ] => Err(Panic::type_error(other.copy(), pos)),
			_ => Err(Panic::invalid_args(args.len() as u32, 1, pos))
		}
	}
}


/// std.bind
#[derive(Trace, Finalize)]
struct Bind;

impl NativeFun for Bind {
	fn name(&self) -> &'static str { "std.bind" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		let args = &runtime.arguments[args_start..];

		match args {
			[ obj, Value::Function(fun) ] => Ok(
				BindImpl {
					obj: obj.copy(),
					fun: fun.copy(),
				}.into()
			),

			[ _, other ] => Err(Panic::type_error(other.copy(), pos)),
			_ => Err(Panic::invalid_args(args.len() as u32, 2, pos))
		}
	}
}


#[derive(Trace, Finalize)]
struct BindImpl {
	obj: Value,
	fun: Function,
}

impl NativeFun for BindImpl {
	fn name(&self) -> &'static str { "std.bind<impl>" }

	fn call(
		&mut self,
		runtime: &mut Runtime,
		_obj: Value,
		args_start: usize,
		pos: SourcePos,
	) -> Result<Value, Panic> {
		runtime.call(self.obj.copy(), &self.fun, args_start, pos)
	}
}
