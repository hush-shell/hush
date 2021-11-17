use gc::{Finalize, Trace};

use crate::fmt;

use super::{
	CallContext,
	Error,
	NativeFun,
	RustFun,
	Panic,
	Value,
};


inventory::submit!{ RustFun::from(Catch) }

#[derive(Trace, Finalize)]
struct Catch;

impl NativeFun for Catch {
	fn name(&self) -> &'static str { "std.catch" }

	fn call(&self, mut context: CallContext) -> Result<Value, Panic> {
		thread_local! {
			pub static PANIC: Value = "panic".into();
		}

		let fun = match context.args() {
			[ Value::Function(fun) ] => fun.copy(),

			[ other ] => return Err(Panic::type_error(other.copy(), "function", context.pos)),
			args => return Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		};

		let result = context.call(
			Value::default(),
			&fun,
			context.args_start + 1
		);

		match result {
			Ok(value) => Ok(value),

			Err(panic) => {
				let description = format!(
					"caught panic: {}",
					fmt::Show(panic, context.interner()),
				);

				Ok(
					Value::from(
						Error::new(
							description.into(),
							PANIC.with(Value::copy),
						)
					)
				)
			}
		}
	}
}
