use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Pop) }

#[derive(Trace, Finalize)]
struct Pop;

impl NativeFun for Pop {
	fn name(&self) -> &'static str { "std.pop" }

	fn call(&self, mut context: CallContext) -> Result<Value, Panic> {
		match context.args_mut() {
			[ Value::Array(ref mut array) ] => {
				let value = array
					.pop()
					.map_err(|_| Panic::empty_collection(context.pos))?;

				Ok(value)
			},

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
