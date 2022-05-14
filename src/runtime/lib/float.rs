use gc::{Finalize, Trace};

use super::{
	CallContext,
	NativeFun,
	RustFun,
	Panic,
	Value,
};


inventory::submit!{ RustFun::from(Float) }

#[derive(Trace, Finalize)]
struct Float;

impl NativeFun for Float {
	fn name(&self) -> &'static str { "std.float" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Float(f) ] => Ok(
				Value::Float(f.copy())
			),

			[ Value::Int(i) ] => Ok(
				Value::Float(i.into())
			),

			[ value @ Value::String(ref string) ] => {
				let parse_error = || Panic::value_error(
					value.copy(),
					"valid integer",
					context.pos.copy()
				);

				let slice = std::str
					::from_utf8(string.as_bytes())
					.map_err(|_| parse_error())?;

				let float: f64 = slice
					.parse()
					.map_err(|_| parse_error())?;

				Ok(Value::from(float))
			}

			[ other ] => Err(Panic::type_error(other.copy(), "int, float or string", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
