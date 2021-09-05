use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Sort) }

#[derive(Trace, Finalize)]
struct Sort;

impl NativeFun for Sort {
	fn name(&self) -> &'static str { "std.sort" }

	fn call(&self, mut context: CallContext) -> Result<Value, Panic> {
		match context.args_mut() {
			[ Value::Array(ref mut array) ] => {
				array.sort();
				Ok(Value::default())
			}

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
