use super::{program, Runtime, Panic, Value};


impl<'a> Runtime<'a> {
	pub(super) fn eval_command_block(
		&mut self,
		block: &'static program::CommandBlock
	) -> Result<Value, Panic> {
		todo!("{:?}", block)
	}
}
