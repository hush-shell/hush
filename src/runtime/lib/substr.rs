use gc::{Finalize, Trace};

use super::{
    CallContext,
    RustFun,
    NativeFun,
    Panic,
    Value,
    Error,
};


inventory::submit! { RustFun::from(Substr) }

#[derive(Trace, Finalize)]
struct Substr;

impl Substr {
    fn substr(string: &[u8], start: i64, len: i64) -> Result<Value, Error> {
        let start = start as usize;
        let end = start + (len as usize);
        let substr = string.get(start..end);
        Ok(substr.into())
    }
}

impl NativeFun for Substr {
    fn name(&self) -> &'static str { "std.substr" }

    fn call(&self, context: CallContext) -> Result<Value, Panic> {
        match context.args() {
            [ Value::String(ref string), Value::Int(start), Value::Int(len) ] => {
                // TODO Panic if indexes out of bounds
                let result = Self::substr(string.as_ref(), *start, *len);
                Ok(result.unwrap_or_else(Into::into))
            },
            [ other, Value::Int(_), Value::Int(_) ] => Err(Panic::type_error(other.copy(), "string", context.pos)),
            [ Value::String(_), other, Value::Int(_) ] => Err(Panic::type_error(other.copy(), "int", context.pos)),
            [ Value::String(_), Value::Int(_), other ] => Err(Panic::type_error(other.copy(), "int", context.pos)),
            args => Err(Panic::invalid_args(args.len() as u32, 3, context.pos))
        }
    }
}

