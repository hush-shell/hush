use gc::{Finalize, GcCell, Trace};

use crate::runtime::value::{CallContext, NativeFun, Value};

use super::{Panic, PipelineErrors, IntoValue};


#[derive(Finalize)]
struct JoinHandle(
	std::thread::JoinHandle<Result<Box<[PipelineErrors]>, Panic>>
);


unsafe impl Trace for JoinHandle {
	gc::unsafe_empty_trace!();
}


#[derive(Trace, Finalize)]
pub struct Join(GcCell<Option<JoinHandle>>);


impl Join {
	pub fn new(handle: std::thread::JoinHandle<Result<Box<[PipelineErrors]>, Panic>>) -> Self {
		Self(
			GcCell::new(
				Some(JoinHandle(handle))
			)
		)
	}
}


impl NativeFun for Join {
	fn name(&self) -> &'static str { "<command>.join" }

	fn call(&self, context: CallContext) -> Result<Value, crate::runtime::Panic> {
		match self.0.borrow_mut().take() {
			Some(JoinHandle(join_handle)) => {
				let result = match join_handle.join() {
					Ok(result) => result,
					Err(error) => std::panic::resume_unwind(error),
				};

				result
					.map(|errors| errors.into_value(context.interner()))
					.map_err(Into::into)
			},

			None => Err(
				crate::runtime::Panic::invalid_join(context.pos),
			)
		}
	}
}
