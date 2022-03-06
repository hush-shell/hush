use gc::{Finalize, Trace};

use crate::runtime::value::Error;

use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Env) }
inventory::submit! { RustFun::from(Export) }

#[derive(Trace, Finalize)]
struct Env;

impl NativeFun for Env {
	fn name(&self) -> &'static str { "std.env" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ Value::String(ref string) ] => Ok(
				std::env
					::var_os(string)
					.map(Value::from)
					.unwrap_or(Value::Nil)
			),

			[ other ] => Err(Panic::type_error(other.copy(), "string", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		}
	}
}

#[derive(Trace, Finalize)]
struct Export;

impl NativeFun for Export {
	fn name(&self) -> &'static str { "std.export" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ k @ Value::String(ref key), v @ Value::String(ref value) ] => {
				let ret = match () {
					() if key.contains(b'=') || key.contains(b'\0') => {
						Error::new("invalid export key".into(), k.copy()).into()
					}

					() if value.contains(b'\0') => {
						Error::new("invalid export value".into(), v.copy()).into()
					}

					() => {
						std::env::set_var(key, value);
						Value::default()
					},
				};

				Ok(ret)
			},

			[ Value::String(_), other ] => Err(Panic::type_error(other.copy(), "string", context.pos)),
			[ other, _ ] => Err(Panic::type_error(other.copy(), "string", context.pos)),

			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos)),
		}
	}
}
