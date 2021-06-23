use super::value::Value;


/// Control flow in the language.
#[derive(Debug)]
pub enum Flow {
	/// Regular flow: follow everything in order.
	Regular(Value),
	/// Return from function.
	Return(Value),
	/// Break from loop.
	Break,
}
