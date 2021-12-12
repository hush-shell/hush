use super::{
	argument,
	Argument,
	ArgExpansion,
	Cursor,
	Checkpoint,
	State,
	Transition,
};


/// The state context for the Expansion state.
/// The Expansion state is generic in the sense that it returns to the previous state once
/// it is finished. Such previous state is the ExpansionContext. Unless an expansion is
/// successfully produced, the cursor will be reset to where it when the expansion context
/// started.
pub(super) trait ExpansionContext {
	/// The transition to make when a expansion has been produced.
	fn produce(self, expansion: ArgExpansion) -> Transition;
	/// The transition to make when no expansion could be parsed.
	/// Yield and rollback to the given checkpoint.
	fn rollback(self, checkpoint: Checkpoint) -> Transition;
	/// Check if a character may be consumed inside expansions.
	fn is_expansion_word(value: u8) -> bool;
}


/// The base state for lexing expansions.
#[derive(Debug)]
pub(super) struct Expansion<C> {
	start: Checkpoint,
	/// Whether to allow recognition of the home expansion.
	allow_home: bool,
	/// Whether the tilde has been consumed for the home expansion.
	tilde_consumed: bool,
	/// The argument context.
	context: C,
}


impl<C> Expansion<C>
where
	C: ExpansionContext,
	State: From<Self>,
{
	pub fn at(cursor: &Cursor, allow_home: bool, context: C) -> Self {
		Self {
			start: cursor.checkpoint(),
			allow_home,
			tilde_consumed: false,
			context,
		}
	}


	pub fn visit(mut self, cursor: &Cursor) -> Transition {
		// Note that we must only allow home expansion in the beggining of the input.
		let allow_home = self.allow_home;
		self.allow_home = false;

		match cursor.peek() {
			// Home expansion start.
			Some(b'~') if allow_home => {
				self.tilde_consumed = true;
				Transition::step(self)
			}

			// Home expansion end.
			Some(b'/') if self.tilde_consumed => {
				self.context.produce(ArgExpansion::Home)
			}

			// Home expansion missing tilde.
			Some(_) if self.tilde_consumed => self.context.rollback(self.start),

			// Star.
			Some(b'*') => {
				self.context.produce(ArgExpansion::Star)
			}

			// Percent.
			Some(b'%') => {
				self.context.produce(ArgExpansion::Percent)
			}

			Some(b'[') => {
				todo!() // char class.
			}

			Some(b'{') => {
				todo!() // range, collection
			}

			// Failed to parse expansion.
			_ => self.context.rollback(self.start)
		}
	}
}


impl From<Expansion<Argument>> for State {
	fn from(state: Expansion<Argument>) -> Self {
		Self::Expansion(state)
	}
}


impl From<Expansion<argument::Word<Argument>>> for State {
	fn from(state: Expansion<argument::Word<Argument>>) -> Self {
		Self::ExpansionWord(state)
	}
}


/// Whether a character is an expansion starter.
pub fn is_start(c: u8) -> bool {
	b"{[~*%".contains(&c)
}
