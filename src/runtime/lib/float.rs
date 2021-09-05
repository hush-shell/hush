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
			[ Value::Int(i) ] => Ok(
				Value::Float(i.into())
			),

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
