use std::io::{self, Write};

use gc::{Finalize, Trace};

use crate::{fmt, symbol};
use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(PrintC) }

#[derive(Trace, Finalize)]
struct PrintC;


impl PrintC {
	fn printc<W: Write>(value: &Value, interner: &symbol::Interner, mut writer: W) -> io::Result<()> {
		match value {
			Value::String(string) => writer.write_all(string.as_ref()),
			Value::Byte(byte) => writer.write_all(&[*byte]),
			value => write!(writer, "{}", fmt::Show(value, interner)),
		}
	}
}


impl NativeFun for PrintC {
	fn name(&self) -> &'static str { "std.printc" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		let stdout = io::stdout();
		let mut stdout = stdout.lock();

		let mut iter = context.args().iter();

		if let Some(value) = iter.next() {
			Self::printc(value, context.interner(), &mut stdout)
				.map_err(|error| Panic::io(error, context.pos.copy()))?;
		}

		for value in iter {
			write!(stdout, "\t")
				.map_err(|error| Panic::io(error, context.pos.copy()))?;

			Self::printc(value, context.interner(), &mut stdout)
				.map_err(|error| Panic::io(error, context.pos.copy()))?;
		}

		//writeln!(stdout)
			//.map_err(|error| Panic::io(error, context.pos))?;

		Ok(Value::default())
	}
}
