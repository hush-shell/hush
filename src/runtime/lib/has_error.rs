use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(HasError) }

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
