use std::ffi::OsStr;

use gc::{Finalize, Trace};

use super::{
	CallContext,
	NativeFun,
	RustFun,
	Panic,
	Value,
};


inventory::submit!{ RustFun::from(Cd) }

#[derive(Trace, Finalize)]
struct Cd;

impl NativeFun for Cd {
	fn name(&self) -> &'static str { "std.cd" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string) ] => Ok(
				std::env
					::set_current_dir(AsRef::<OsStr>::as_ref(string))
					.into()
			),

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
