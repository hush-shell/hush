use std::fmt::Display as _;

use super::{lexer, Capture, FrameInfo, SlotIx};
use crate::{
	fmt::{self, Display, Indentation},
	term::color
};


impl std::fmt::Display for SlotIx {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		color::Fg(color::Green, '#').fmt(f)?;
		color::Fg(color::Green, self.0).fmt(f)
	}
}


impl<'a> Display<'a> for FrameInfo {
	type Context = Option<Indentation>;

	fn fmt(&self, f: &mut std::fmt::Formatter, context: Self::Context) -> std::fmt::Result {
		#[derive(Clone)]
		enum Slot {
			Regular,
			Capture { from: SlotIx },
		}

		let mut slots: Box<[(SlotIx, Slot)]> = std::iter
			::repeat(Slot::Regular)
			.take(self.slots.0 as usize)
			.enumerate()
			.map(
				|(ix, slot)| (SlotIx(ix as u32), slot)
			)
			.collect();

		for Capture { from, to } in self.captures.iter().copied() {
			slots[to.0 as usize].1 = Slot::Capture { from };
		}

		fmt::sep_by(
			slots.iter(),
			f,
			|(slot_ix, slot), f| {
				if let Some(indent) = context {
					indent.fmt(f)?;
				} else {
					" ".fmt(f)?;
				}

				lexer::Keyword::Let.fmt(f)?;
				" ".fmt(f)?;
				slot_ix.fmt(f)?;
				": ".fmt(f)?;

				match slot {
					Slot::Regular => color::Fg(color::Blue, "auto").fmt(f)?,
					Slot::Capture { from } => {
						color::Fg(color::Blue, "capture").fmt(f)?;
						" ".fmt(f)?;
						from.fmt(f)?;
					}
				}

				if Some(*slot_ix) == self.self_slot {
					color::Fg(color::Blue, " self").fmt(f)?;
				}

				Ok(())
			},
			if context.is_some() { "\n" } else { ";" },
		)
	}
}
