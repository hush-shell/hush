use crate::{
	fmt::{self, Display},
	symbol,
};
use super::{
	Array,
	Dict,
	Float,
	Function,
	HushFun,
	RustFun,
	Str,
	Value,
};


impl std::fmt::Display for RustFun {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.name())
	}
}


impl<'a> Display<'a> for HushFun {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		write!(f, "function<{}>", fmt::Show(&self.pos, context))
	}
}


impl<'a> Display<'a> for Function {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Hush(fun) => write!(f, "{}", fmt::Show(fun, context)),
			Self::Rust(fun) => write!(f, "{}", fun),
		}
	}
}


impl std::fmt::Display for Float {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}


impl<'a> Display<'a> for Array {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		write!(f, "[")?;

		for item in self.borrow().iter() {
			write!(f, "{},", fmt::Show(item, context))?;
		}

		write!(f, "]")
	}
}


impl<'a> Display<'a> for Dict {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		write!(f, "@[")?;

		for (k, v) in self.borrow().iter() {
			write!(
				f,
				"{}:{},",
				fmt::Show(k, context),
				fmt::Show(v, context)
			)?;
		}

		write!(f, "]")
	}
}


impl std::fmt::Display for Str {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "\"{}\"", String::from_utf8_lossy(self.as_ref()).escape_debug())
	}
}


impl<'a> Display<'a> for Value {
	type Context = &'a symbol::Interner;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		match self {
			Self::Nil => write!(f, "nil"),
			Self::Bool(b) => write!(f, "{}", b),
			Self::Int(int) => write!(f, "{}", int),
			Self::Float(float) => write!(f, "{}", float),
			Self::Byte(byte) => write!(f, "{}", *byte as char),
			Self::String(string) => write!(f, "{}", string),
			Self::Array(array) => write!(f, "{}", fmt::Show(array, context)),
			Self::Dict(dict) => write!(f, "{}", fmt::Show(dict, context)),
			Self::Function(fun) => write!(f, "{}", fmt::Show(fun, context)),
			Self::Error(error) => write!(f, "error: {}", error.description),
		}
	}
}
