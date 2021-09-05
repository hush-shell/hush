use std::path::PathBuf;

use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Cwd) }

#[derive(Trace, Finalize)]
struct Cwd;

impl NativeFun for Cwd {
	fn name(&self) -> &'static str { "std.cwd" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		let args = context.args();
		if !args.is_empty() {
			return Err(Panic::invalid_args(args.len() as u32, 0, context.pos));
		}

		Ok(
			std::env
				::current_dir()
				.map(PathBuf::into_os_string)
				.into()
		)
	}
}
