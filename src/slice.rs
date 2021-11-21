/// Extension trait for generalizing str::split_once to slices.
pub trait SplitOnce {
	type Item;

	/// Splits the slice on the first occurrence of the specified delimiter and returns
	/// prefix before delimiter and suffix after delimiter.
	fn split_once<P>(&self, predicate: P) -> Option<(&[Self::Item], &[Self::Item])>
	where
		P: FnMut(&Self::Item) -> bool;
}


impl<T> SplitOnce for [T] {
	type Item = T;

	fn split_once<P>(&self, predicate: P) -> Option<(&[T], &[T])>
	where
		P: FnMut(&T) -> bool,
	{
		self
			.iter()
			.position(predicate)
			.map(
				|ix| {
					let (prefix, suffix) = self.split_at(ix);
					(prefix, &suffix[1..])
				}
			)
	}
}
