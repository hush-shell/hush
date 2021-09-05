use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Push) }

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
