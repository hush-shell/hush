use std::fmt::{self, Display, Debug};

use super::{Array, Dict, Float, Function, HushFun, RustFun, Value};


impl Debug for RustFun {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.name())
	}
}


impl Display for RustFun {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.name())
	}
}


impl Display for HushFun {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "function<{}>", self.pos)
	}
}


impl Display for Function {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Hush(fun) => write!(f, "{}", fun),
			Self::Rust(fun) => write!(f, "{}", fun),
		}
	}
}


impl Display for Float {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}


impl Display for Array {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "[")?;

		for item in self.borrow().iter() {
			write!(f, "{},", item)?;
		}

		write!(f, "]")
	}
}


impl Display for Dict {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "@[")?;

		for (k, v) in self.borrow().iter() {
			write!(f, "{}:{},", k, v)?;
		}

		write!(f, "]")
	}
}


impl Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Nil => write!(f, "nil"),
			Self::Bool(b) => write!(f, "{}", b),
			Self::Int(int) => write!(f, "{}", int),
			Self::Float(float) => write!(f, "{}", float),
			Self::Byte(byte) => write!(f, "{}", *byte as char),
			Self::String(string) => write!(f, "{}", String::from_utf8_lossy(string.as_ref()).escape_debug()),
			Self::Array(array) => write!(f, "{}", array),
			Self::Dict(dict) => write!(f, "{}", dict),
			Self::Function(fun) => write!(f, "{}", fun),
			Self::Error() => todo!(),
		}
	}
}
