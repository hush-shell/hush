use gc::{Finalize, Trace};

use super::{
	CallContext,
	NativeFun,
	RustFun,
	Panic,
	Value,
};


inventory::submit!{ RustFun::from(Assert) }

#[derive(Trace, Finalize)]
struct Assert;

impl NativeFun for Assert {
	fn name(&self) -> &'static str { "std.assert" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::Bool(true) ] => Ok(Value::default()),
			[ Value::Bool(false) ] => Err(Panic::assertion_failed(context.pos)),

			[ other ] => Err(Panic::type_error(other.copy(), "bool", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
