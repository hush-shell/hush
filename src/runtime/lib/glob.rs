use gc::{Finalize, Trace};

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
	Error,
};


inventory::submit! { RustFun::from(Glob) }

#[derive(Trace, Finalize)]
struct Glob;

impl Glob {
	fn glob(pattern: &[u8]) -> Result<Value, Error> {
		let pattern = std::str::from_utf8(pattern).map_err(|_| Error::new("Invalid pattern".into(), Value::default()))?;
		let paths = glob::glob(pattern).map_err(|error| Error::new("Invalid pattern".into(), error.msg.into()))?;
		let paths: Vec<Value> = paths
			.map(|result| result
				 .map(|path| Value::String(path.into()))
				 .map_err(|error| error.into_error().into())
			)
			.collect::<Result<_, Error>>()?;
		Ok(paths.into())
	}
}

impl NativeFun for Glob {
	fn name(&self) -> &'static str { "std.glob" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
		    [ Value::String(ref string) ] => {
				let result = Self::glob(string.as_ref());
				Ok(result.unwrap_or_else(Into::into))
			},
		    [ other ] => Err(Panic::type_error(other.copy(), "string", context.pos)),
		    args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}
