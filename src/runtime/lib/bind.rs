use gc::{Finalize, Trace};

use super::{
	CallContext,
	Function,
	NativeFun,
	RustFun,
	Panic,
	Value,
};


inventory::submit!{ RustFun::from(Bind) }

#[derive(Trace, Finalize)]
struct Bind;

impl NativeFun for Bind {
	fn name(&self) -> &'static str { "std.bind" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		match context.args() {
			[ obj, Value::Function(fun) ] => Ok(
				BindImpl {
					obj: obj.copy(),
					function: fun.copy(),
				}.into()
			),

			[ _, other ] => Err(Panic::type_error(other.copy(), "function", context.pos)),
			args => Err(Panic::invalid_args(args.len() as u32, 2, context.pos))
		}
	}
}


#[derive(Trace, Finalize)]
struct BindImpl {
	obj: Value,
	function: Function,
}

impl NativeFun for BindImpl {
	fn name(&self) -> &'static str { "std.bind<impl>" }

	fn call(&self, mut context: CallContext) -> Result<Value, Panic> {
		context.call(self.obj.copy(), &self.function, context.args_start)
	}
}
