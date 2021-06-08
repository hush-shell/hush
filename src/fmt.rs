use std::fmt::Write;

/// A Display-like trait that takes an additional context when formatting.
/// This is needed to have access to the string interner when formating the AST or error
/// messages.
pub trait Display<'a> {
	/// The format context.
	type Context: 'a;

	fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: Self::Context) -> std::fmt::Result;
}


impl<'a, T> Display<'a> for &T
where
	T: Display<'a>,
{
	type Context = T::Context;

	fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: Self::Context) -> std::fmt::Result {
		(*self).fmt(f, context)
	}
}


impl<'a, T> Display<'a> for &mut T
where
	T: Display<'a>,
{
	type Context = T::Context;

	fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: Self::Context) -> std::fmt::Result {
		(&**self).fmt(f, context)
	}
}


/// An adapter to use std::fmt::Display with the contextual Display.
#[derive(Debug)]
pub struct Show<T, C>(pub T, pub C);


impl<'a, T, C> std::fmt::Display for Show<T, C>
where
	T: Display<'a, Context = C>,
	C: Copy,
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.0.fmt(f, self.1)
	}
}



/// An indentation level. Each level corresponds to one tabulation character.
#[derive(Debug, Default, Copy, Clone)]
pub struct Indentation(pub u8);


impl Indentation {
	/// Increase the identation level by one.
	pub fn increase(self) -> Self {
		Indentation(self.0 + 1)
	}
}


impl std::fmt::Display for Indentation {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		for _ in 0 .. self.0 {
			f.write_char('\t')?;
		}

		Ok(())
	}
}
