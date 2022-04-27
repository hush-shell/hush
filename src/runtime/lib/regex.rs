use std::{rc::Rc, collections::HashMap, borrow::Cow};

use gc::{Finalize, Trace};
use regex::bytes::Regex;

use super::{
	Error,
	CallContext,
	Dict,
	RustFun,
	NativeFun,
	Panic,
	Str,
	Value,
};


inventory::submit! { RustFun::from(StdRegex) }

#[derive(Trace, Finalize)]
struct StdRegex;

impl StdRegex {
	fn build(pattern: &[u8]) -> Value {
		let pattern = match std::str::from_utf8(pattern) {
			Ok(pattern) => pattern,
			Err(error) => return Error::new("invalid regex".into(), error.to_string().into()).into(),
		};

		let pattern = match Regex::new(pattern) {
			Ok(pattern) => Rc::new(pattern),
			Err(error) => return Error::new("invalid regex".into(), error.to_string().into()).into(),
		};

		thread_local! {
			pub static MATCH: Value = "match".into();
			pub static SPLIT: Value = "split".into();
			pub static REPLACE: Value = "replace".into();
		}

		let mut dict = HashMap::new();

		MATCH.with(
			|name| dict.insert(name.copy(), RegexMatchImpl { pattern: pattern.clone() }.into())
		);

		SPLIT.with(
			|split| dict.insert(split.copy(), RegexSplitImpl { pattern: pattern.clone() }.into())
		);

		REPLACE.with(
			|replace| dict.insert(replace.copy(), RegexReplaceImpl { pattern: pattern.clone() }.into())
		);

		Dict::new(dict).into()
	}
}

impl NativeFun for StdRegex {
	fn name(&self) -> &'static str { "std.regex" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string) ] => Ok(Self::build(string.as_ref())),

			[ other ] => Err(Panic::type_error(other.copy(), "string", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}


#[derive(Finalize)]
struct RegexMatchImpl {
	pattern: Rc<Regex>,
}

/// RegexMatchImpl has no garbage-collected fields.
unsafe impl Trace for RegexMatchImpl {
	gc::unsafe_empty_trace!();
}

impl NativeFun for RegexMatchImpl {
	fn name(&self) -> &'static str { "std.regex<match>" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string) ] => Ok(self.pattern.is_match(string.as_ref()).into()),

			[ other ] => Err(Panic::type_error(other.copy(), "string", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}

#[derive(Finalize)]
struct RegexSplitImpl {
	pattern: Rc<Regex>,
}

/// RegexSplitImpl has no garbage-collected fields.
unsafe impl Trace for RegexSplitImpl {
	gc::unsafe_empty_trace!();
}

impl NativeFun for RegexSplitImpl {
	fn name(&self) -> &'static str { "std.regex<split>" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string) ] => Ok(
				self.pattern
					.split(string.as_ref())
					.map(Str::from)
					.map(Value::from)
					.collect::<Vec<_>>()
					.into()
			),

			[ other ] => Err(Panic::type_error(other.copy(), "string", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}

#[derive(Finalize)]
struct RegexReplaceImpl {
	pattern: Rc<Regex>,
}

/// RegexReplaceImpl has no garbage-collected fields.
unsafe impl Trace for RegexReplaceImpl {
	gc::unsafe_empty_trace!();
}

impl NativeFun for RegexReplaceImpl {
	fn name(&self) -> &'static str { "std.regex<replace>" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ value @ Value::String(ref string), Value::String(ref replace) ] => Ok(
				match self.pattern.replace_all(string.as_ref(), replace.as_bytes()) {
					Cow::Borrowed(_) => value.copy(),
					Cow::Owned(value) => Str::from(value).into(),
				}
			),

			[ Value::String(_), other ] | [ other, _ ] => {
				Err(Panic::type_error(other.copy(), "string", context.pos))
			},

			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos))
		}
	}
}
