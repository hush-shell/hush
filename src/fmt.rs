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


/// A ToString-like trait that takes an additional context when formatting.
/// This is needed to have access to the string interner when formating the AST or error
/// messages.
pub trait FmtString<'a> {
	/// The format context.
	type Context: 'a;

	fn fmt_string(&self, context: Self::Context) -> String;
}


impl<'a, T> FmtString<'a> for T
where
	T: Display<'a>,
	T::Context: Copy,
{
	type Context = T::Context;

	fn fmt_string(&self, context: Self::Context) -> String {
		let mut string = String::new();
		write!(string, "{}", Show(self, context))
			.expect("a Display implementation returned an error unexpectedly");
		string
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


/// Format a sequence of items with a separator.
pub fn sep_by<T, I, F, S>(
	mut iter: I,
	f: &mut std::fmt::Formatter,
	mut format: F,
	separator: S,
) -> std::fmt::Result
where
	I: Iterator<Item = T>,
	F: FnMut(T, &mut std::fmt::Formatter) -> std::fmt::Result,
	S: std::fmt::Display,
{
	if let Some(item) = iter.next() {
		format(item, f)?;
	}

	for item in iter {
		separator.fmt(f)?;
		format(item, f)?;
	}

	Ok(())
}
