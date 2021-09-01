mod util;

use std::{
	collections::HashMap,
	io::{self, Write},
	path::Path
};

use gc::{Finalize, GcCell, Trace};

use crate::{
	fmt::{self, FmtString},
	syntax,
	semantic,
	symbol::{self, Symbol}
};
use super::{
	keys,
	Array,
	CallContext,
	Dict,
	Error,
	Float,
	Function,
	NativeFun,
	Panic,
	Str,
	Value,
};


/// Instantiate the stdlib.
pub fn new() -> Value {
	let mut dict = HashMap::new();

	dict.insert("assert".into(), Assert.into());
	dict.insert("bind".into(), Bind.into());
	dict.insert("cd".into(), Cd.into());
	dict.insert("contains".into(), Contains.into());
	dict.insert("cwd".into(), Cwd.into());
	dict.insert("env".into(), Env.into());
	dict.insert("error".into(), ErrorFun.into());
	dict.insert("has_error".into(), HasError.into());
	dict.insert("import".into(), Import.into());
	dict.insert("is_empty".into(), IsEmpty.into());
	dict.insert("iter".into(), Iter.into());
	dict.insert("len".into(), Length.into());
	dict.insert("pop".into(), Pop.into());
	dict.insert("print".into(), Print.into());
	dict.insert("push".into(), Push.into());
	dict.insert("range".into(), Range.into());
	dict.insert("sort".into(), Sort.into());
	dict.insert("split".into(), Split.into());
	dict.insert("trim".into(), Trim.into());
	dict.insert("to_string".into(), ToString.into());
	dict.insert("type".into(), Type.into());
	dict.insert("replace".into(), Replace.into());

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

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		let stdout = io::stdout();
		let mut stdout = stdout.lock();

		let mut iter = context.args().iter();

		if let Some(value) = iter.next() {
			Self::print(value, context.interner(), &mut stdout)
				.map_err(|error| Panic::io(error, context.pos.copy()))?;
		}

		for value in iter {
			write!(stdout, "\t")
				.map_err(|error| Panic::io(error, context.pos.copy()))?;

			Self::print(value, context.interner(), &mut stdout)
				.map_err(|error| Panic::io(error, context.pos.copy()))?;
		}

		writeln!(stdout)
			.map_err(|error| Panic::io(error, context.pos))?;

		Ok(Value::default())
	}
}


/// std.type
#[derive(Trace, Finalize)]
struct Type;

impl NativeFun for Type {
	fn name(&self) -> &'static str { "std.type" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
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

		let typename = match context.args() {
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
			args @ [] | args @ [_, _, ..] => return Err(
				Panic::invalid_args(args.len() as u32, 1, context.pos)
			),
		};

		Ok(typename.with(Value::copy))
	}
}


/// std.length
#[derive(Trace, Finalize)]
struct Length;

impl NativeFun for Length {
	fn name(&self) -> &'static str { "std.len" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Array(ref array) ] => Ok(Value::Int(array.len())),
			[ Value::Dict(ref dict) ] => Ok(Value::Int(dict.len())),
			[ Value::String(ref string) ] => Ok(Value::Int(string.len() as i64)),
			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}


/// std.iter
#[derive(Trace, Finalize)]
struct Iter;

impl NativeFun for Iter {
	fn name(&self) -> &'static str { "std.iter" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Array(ref array) ] => Ok(
				IterImpl::Array {
					array: array.copy(),
					ix: GcCell::new(0),
				}.into()
			),

			[ Value::Dict(ref dict) ] => Ok(
				IterImpl::Dict {
					entries: GcCell::new(
						dict
							.borrow()
							.iter()
							.map(|(k, v)| (k.copy(), v.copy()))
							.collect()
					)
				}.into()
			),

			[ Value::String(ref string) ] => Ok(
				IterImpl::String {
					string: string.copy(),
					ix: GcCell::new(0),
				}.into()
			),

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}


#[derive(Trace, Finalize)]
enum IterImpl {
	Array {
		array: Array,
		ix: GcCell<i64>,
	},
	String {
		string: Str,
		ix: GcCell<i64>,
	},
	Dict {
		entries: GcCell<Vec<(Value, Value)>>,
	}
}

impl NativeFun for IterImpl {
	fn name(&self) -> &'static str { "std.iter<impl>" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		let args = context.args();
		if !args.is_empty() {
			return Err(Panic::invalid_args(args.len() as u32, 0, context.pos));
		}

		let mut iteration = HashMap::new();

		let next = match self {
			IterImpl::Array { array, ix } => {
				let mut ix = ix.borrow_mut();
				if let Ok(value) = array.index(*ix) {
					*ix += 1;
					Some(value)
				} else {
					None
				}
			}

			IterImpl::String { string, ix } => {
				let mut ix = ix.borrow_mut();
				if let Ok(value) = string.index(*ix) {
					*ix += 1;
					Some(value)
				} else {
					None
				}
			}

			IterImpl::Dict { entries } => entries
				.borrow_mut()
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

	fn call(&self, mut context: CallContext) -> Result<Value, Panic> {
		match context.args_mut() {
			[ Value::Array(ref mut array), value ] => {
				array.push(value.copy());
				Ok(Value::Nil)
			},

			[ other, _ ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos))
		}
	}
}


/// std.pop
#[derive(Trace, Finalize)]
struct Pop;

impl NativeFun for Pop {
	fn name(&self) -> &'static str { "std.pop" }

	fn call(&self, mut context: CallContext) -> Result<Value, Panic> {
		match context.args_mut() {
			[ Value::Array(ref mut array) ] => {
				let value = array
					.pop()
					.map_err(|_| Panic::empty_collection(context.pos))?;

				Ok(value)
			},

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}


/// std.is_empty
#[derive(Trace, Finalize)]
struct IsEmpty;

impl NativeFun for IsEmpty {
	fn name(&self) -> &'static str { "std.is_empty" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Array(ref array) ] => Ok(array.is_empty().into()),

			[ Value::Dict(ref dict) ] => Ok(dict.is_empty().into()),

			[ Value::String(ref string) ] => Ok(string.is_empty().into()),

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}


/// std.error
#[derive(Trace, Finalize)]
struct ErrorFun;

impl NativeFun for ErrorFun {
	fn name(&self) -> &'static str { "std.error" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string), context ] => Ok(
				Error
					::new(string.copy(), context.copy())
					.into()
			),

			[ other, _ ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos))
		}
	}
}


/// std.range
#[derive(Trace, Finalize)]
struct Range;

impl NativeFun for Range {
	fn name(&self) -> &'static str { "std.range" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ from, to, step ] => {
				let numbers = util::Numbers
					::promote([from.copy(), to.copy(), step.copy()])
					.map_err(|value| Panic::type_error(value, context.pos))?;

				Ok(
					match numbers {
						util::Numbers::Ints([ from, to, step ]) => RangeImpl {
							from: GcCell::new(from),
							to,
							step
						}.into(),

						util::Numbers::Floats([ from, to, step ]) => RangeImpl {
							from: GcCell::new(from),
							to,
							step
						}.into(),
					}
				)
			},

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 3, context.pos))
		}
	}
}


#[derive(Trace, Finalize)]
struct RangeImpl<T: 'static> {
	from: GcCell<T>,
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

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		let args = context.args();
		if !args.is_empty() {
			return Err(Panic::invalid_args(args.len() as u32, 0, context.pos));
		}

		let mut from = self.from.borrow_mut();
		let mut iteration = HashMap::new();

		let finished =
			if self.step > T::default() { // Step is positive.
				*from >= self.to
			} else { // Step is negative.
				*from <= self.to
			};

		let next = if finished {
			None
		} else {
			let value = from.clone();
			*from = from.clone() + self.step.clone();
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

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string) ] => Ok(
				std::env
					::var_os(string)
					.map(Value::from)
					.unwrap_or(Value::Nil)
			),

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
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

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ value ] => Ok(Self::has_error(value).into()),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}


/// std.to_string
#[derive(Trace, Finalize)]
struct ToString;

impl NativeFun for ToString {
	fn name(&self) -> &'static str { "std.to_string" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string) ] => Ok(string.copy().into()),
			[ value ] => Ok(value.fmt_string(context.interner()).into()),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}


/// std.cd
#[derive(Trace, Finalize)]
struct Cd;

impl NativeFun for Cd {
	fn name(&self) -> &'static str { "std.cd" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		use std::ffi::OsStr;

		match context.args() {
			[ Value::String(ref string) ] => Ok(
				std::env
					::set_current_dir(AsRef::<OsStr>::as_ref(string))
					.into()
			),

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}


/// std.cwd
#[derive(Trace, Finalize)]
struct Cwd;

impl NativeFun for Cwd {
	fn name(&self) -> &'static str { "std.cwd" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		use std::path::PathBuf;

		let args = context.args();
		if !args.is_empty() {
			return Err(Panic::invalid_args(args.len() as u32, 0, context.pos));
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

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Bool(true) ] => Ok(Value::default()),
			[ Value::Bool(false) ] => Err(Panic::assertion_failed(context.pos)),

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}


/// std.bind
#[derive(Trace, Finalize)]
struct Bind;

impl NativeFun for Bind {
	fn name(&self) -> &'static str { "std.bind" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ obj, Value::Function(fun) ] => Ok(
				BindImpl {
					obj: obj.copy(),
					function: fun.copy(),
				}.into()
			),

			[ _, other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos))
		}
	}
}


#[derive(Trace, Finalize)]
struct BindImpl {
	obj: Value,
	function: Function,
}

impl NativeFun for BindImpl {
	fn name(&self) -> &'static str { "std.bind<impl>" }

	fn call(&self, mut context: CallContext) -> Result<Value, Panic> {
		context.call(self.obj.copy(), &self.function, context.args_start)
	}
}


/// std.contains
#[derive(Trace, Finalize)]
struct Contains;

impl NativeFun for Contains {
	fn name(&self) -> &'static str { "std.contains" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Array(ref array), item ] => Ok(array.contains(item).into()),

			[ Value::Dict(ref dict), key ] => Ok(dict.contains(key).into()),

			[ Value::String(ref string), Value::Byte(byte) ] => Ok(string.contains(*byte).into()),
			[ Value::String(_), other ] => Err(Panic::type_error(other.copy(), context.pos)),

			[ other, _ ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos))
		}
	}
}


/// std.sort
#[derive(Trace, Finalize)]
struct Sort;

impl NativeFun for Sort {
	fn name(&self) -> &'static str { "std.sort" }

	fn call(&self, mut context: CallContext) -> Result<Value, Panic> {
		match context.args_mut() {
			[ Value::Array(ref mut array) ] => {
				array.sort();
				Ok(Value::default())
			}

			[ other, _ ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos))
		}
	}
}


/// std.split
#[derive(Trace, Finalize)]
struct Split;

impl NativeFun for Split {
	fn name(&self) -> &'static str { "std.split" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		use bstr::ByteSlice;

		match context.args() {
			[ Value::String(ref string), Value::String(ref pattern) ] => Ok(
				string
					.as_bytes()
					.split_str(pattern)
					.map(Value::from)
					.collect::<Vec<Value>>()
					.into()
			),

			[ Value::String(_), other ] => Err(Panic::type_error(other.copy(), context.pos)),
			[ other, _ ] => Err(Panic::type_error(other.copy(), context.pos)),

			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos))
		}
	}
}


/// std.trim
#[derive(Trace, Finalize)]
struct Trim;

impl NativeFun for Trim {
	fn name(&self) -> &'static str { "std.trim" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		use bstr::ByteSlice;

		match context.args() {
			[ Value::String(ref string) ] => Ok(
				string
					.as_bytes()
					.trim()
					.into()
			),

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),

			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}


/// std.replace
#[derive(Trace, Finalize)]
struct Replace;

impl NativeFun for Replace {
	fn name(&self) -> &'static str { "std.replace" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		use bstr::ByteSlice;

		match context.args() {
			[ Value::String(ref string), Value::String(ref pattern), Value::String(ref replace) ] => Ok(
				Str::from(
					string
						.as_bytes()
						.replace(pattern, replace)
				).into()
			),

			[ Value::String(_), Value::String(_), other ] => Err(Panic::type_error(other.copy(), context.pos)),
			[ Value::String(_), other, _ ] => Err(Panic::type_error(other.copy(), context.pos)),
			[ other, _, _ ] => Err(Panic::type_error(other.copy(), context.pos)),

			args => Err(Panic::invalid_args(args.len() as u32, 3, context.pos))
		}
	}
}


/// std.import
#[derive(Trace, Finalize)]
struct Import;

impl Import {
	fn import(module_path: &Path, mut context: CallContext) -> Result<Value, Panic> {
		let path = Self
			::resolve_path(
				module_path,
				context.pos.path,
				context.runtime.interner_mut()
			)
			.map_err(
				|error| Panic::io(error, context.pos.copy())
			)?;

		match context.runtime.modules.get(&path) {
			Some(module) => Ok(module.copy()), // Don't reload module if cached.
			None => {
				let module = Self::load(path, &mut context)?;
				context.runtime.modules.insert(path, module.copy());
				Ok(module)
			}
		}
	}


	fn resolve_path(
		target_path: &Path,
		current_path: Symbol,
		interner: &mut symbol::Interner,
	) -> io::Result<Symbol> {
		use std::{
			path::PathBuf,
			ffi::OsStr,
			os::unix::ffi::OsStrExt,
		};

		let mut path_buf = PathBuf::from(
			OsStr::from_bytes(
				interner
					.resolve(current_path)
					.expect("failed to resolve symbol")
			).to_owned()
		);
		path_buf.pop(); // Remove the file name.
		path_buf.push(target_path);

		let path = path_buf.canonicalize()?;

		let path_symbol = interner.get_or_intern(
			path
				.as_os_str()
				.as_bytes()
		);

		Ok(path_symbol)
	}


	fn load(path: Symbol, context: &mut CallContext) -> Result<Value, Panic> {
		// Load file.
		let source = syntax::Source
			::from_path(
				path,
				context.runtime.interner_mut()
			).map_err(
				|error| Panic::io(error, context.pos.copy())
			)?;

		// Syntax.
		let syntactic_analysis = syntax::Analysis::analyze(
			source,
			context.runtime.interner_mut()
		);
		let has_syntax_errors = !syntactic_analysis.is_ok();

		if has_syntax_errors {
			eprint!("{}", fmt::Show(
				syntactic_analysis.errors,
				syntax::AnalysisDisplayContext {
					max_errors: Some(20),
					interner: context.runtime.interner(),
				}
			));
			return Err(Panic::import_failed(path, context.pos.copy()));
		}

		// Semantics.
		let program = semantic::Analyzer
			::analyze(
				syntactic_analysis.ast, context.runtime.interner_mut()
			)
			.map_err(
				|errors| {
					eprint!("{}", fmt::Show(
						errors,
						semantic::ErrorsDisplayContext {
							max_errors: Some(20),
							interner: context.runtime.interner(),
						}
					));

					Panic::import_failed(path, context.pos.copy())
				}
			)?;

		// Eval.
		let program = Box::leak(Box::new(program));
		context.runtime.eval(program)
	}
}

impl NativeFun for Import {
	fn name(&self) -> &'static str { "std.import" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		let path = match context.args() {
			[ Value::String(ref string) ] => Path::new(string).to_owned(),

			[ other ] => return Err(Panic::type_error(other.copy(), context.pos)),

			args => return Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		};

		Self::import(&path, context)
	}
}
