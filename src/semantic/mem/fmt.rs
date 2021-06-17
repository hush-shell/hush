use std::fmt::Display as _;

use super::{lexer, FrameInfo, SlotIx, SlotKind};
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
		let iter = self.slots
			.iter()
			.enumerate()
			.map(
				|(ix, slot)| (SlotIx(ix as u32), slot)
			);

		fmt::sep_by(
			iter,
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
					SlotKind::Regular => color::Fg(color::Blue, "auto").fmt(f),
					SlotKind::Closed => color::Fg(color::Blue, "closed").fmt(f),
					SlotKind::Capture { from } => {
						color::Fg(color::Blue, "capture").fmt(f)?;
						" ".fmt(f)?;
						from.fmt(f)
					}
				}
			},
			if context.is_some() { "\n" } else { ";" },
		)
	}
}
