use std::io::{self, Write};

use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Read) }

#[derive(Trace, Finalize)]
struct Read;


impl Read {
	fn read() -> io::Result<Value> {
		let mut input = String::new();

		io::stdin()
			.read_line(&mut input)
			.map(|_| input.into())
	}
}


impl NativeFun for Read {
	fn name(&self) -> &'static str { "std.read" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ ] => Self::read()
				.map_err(|error| Panic::io(error, context.pos)),

			[ Value::String(ref string) ] => {
				let stdout = io::stdout();
				let mut stdout = stdout.lock();

				stdout
					.write_all(string.as_ref())
					.map_err(|error| Panic::io(error, context.pos.copy()))?;

				stdout
					.flush()
					.map_err(|error| Panic::io(error, context.pos.copy()))?;

				Self::read()
					.map_err(|error| Panic::io(error, context.pos))
			},

			[ other ] => Err(Panic::type_error(other.copy(), context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
