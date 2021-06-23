use std::{
	io,
	fmt::{self, Display},
};

use super::{Value, SourcePos};


#[derive(Debug)]
pub enum Panic {
	StackOverflow { pos: SourcePos },
	DivisionByZero { pos: SourcePos },
	IndexOutOfBounds {
		index: Value,
		pos: SourcePos,
	},
	InvalidCall {
		function: Value,
		pos: SourcePos
	},
	MissingParameters { pos: SourcePos },
	InvalidCondition {
		value: Value,
		pos: SourcePos,
	},
	InvalidOperand {
		value: Value,
		pos: SourcePos,
	},
	Io(io::Error),
}


impl Panic {
	pub fn stack_overflow(pos: SourcePos) -> Self {
		Self::StackOverflow { pos }
	}


	pub fn invalid_call(function: Value, pos: SourcePos) -> Self {
		Self::InvalidCall { function, pos }
	}


	pub fn index_out_of_bounds(index: Value, pos: SourcePos) -> Self {
		Self::IndexOutOfBounds { index, pos }
	}


	pub fn invalid_condition(value: Value, pos: SourcePos) -> Self {
		Self::InvalidCondition { value, pos }
	}


	pub fn invalid_operand(value: Value, pos: SourcePos) -> Self {
		Self::InvalidOperand { value, pos }
	}
}


impl From<io::Error> for Panic {
	fn from(error: io::Error) -> Self {
		Self::Io(error)
	}
}


impl Display for Panic {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self) // TODO
	}
}


impl std::error::Error for Panic { }
