use gc::{Finalize, Trace};

use super::{
	CallContext,
	NativeFun,
	RustFun,
	Panic,
	Value,
};


inventory::submit!{ RustFun::from(Int) }

#[derive(Trace, Finalize)]
struct Int;

impl NativeFun for Int {
	fn name(&self) -> &'static str { "std.int" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Int(i) ] => Ok(
				Value::Int(*i)
			),

			[ Value::Float(f) ] => Ok(
				Value::Int(f.into())
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

				let int: i64 = slice
					.parse()
					.map_err(|_| parse_error())?;

				Ok(Value::from(int))
			}

			[ other ] => Err(Panic::type_error(other.copy(), "int, float or string", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
