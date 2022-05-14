use std::convert::TryFrom;

use gc::{Finalize, Trace};

use super::{
	CallContext,
	NativeFun,
	RustFun,
	Panic,
	Value,
};


inventory::submit!{ RustFun::from(Exit) }

#[derive(Trace, Finalize)]
struct Exit;

impl NativeFun for Exit {
	fn name(&self) -> &'static str { "std.exit" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ val @ Value::Int(i) ] => {
				let code = u8::try_from(*i)
					.map_err(|_| Panic::value_error(val.copy(), "valid exit code", context.pos.copy()))?;

				std::process::exit(code.into())
			}

			[ other ] => Err(Panic::type_error(other.copy(), "int", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
