mod fmt;

use string_interner::{DefaultSymbol, StringInterner, Symbol as DefaultSymbolExt};


/// A symbol is a reference to an identifier stored in the string interner.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Symbol(DefaultSymbol);


/// The default symbol is a dummy symbol, which will yield "<invalid symbol>" when
/// resolved.
impl Default for Symbol {
	fn default() -> Self {
		Self(
			DefaultSymbol
				::try_from_usize(0)
				.expect("invalid default symbol")
		)
	}
}


impl From<Symbol> for usize {
	fn from(symbol: Symbol) -> usize {
		symbol.0.to_usize()
	}
}


/// A string interner, used to store identifiers.
#[derive(Debug)]
pub struct Interner(StringInterner);


impl Interner {
	/// Create a new interner. Please note that this allocates memory even if no symbols are
	/// inserted.
	pub fn new() -> Self {
		let mut interner = StringInterner::new();
		interner.get_or_intern("<invalid symbol>");
		Self(interner)
	}


	/// Get the symbol for a string.
	pub fn get<T>(&self, string: T) -> Option<Symbol>
	where
		T: AsRef<str>,
	{
		self.0
			.get(string)
			.map(Symbol)
	}


	/// Get the symbol for a string. The string is interned if needed.
	pub fn get_or_intern<T>(&mut self, string: T) -> Symbol
	where
		T: AsRef<str>,
	{
		Symbol(self.0.get_or_intern(string))
	}


	/// Resolve the string for a symbol.
	pub fn resolve(&self, symbol: Symbol) -> Option<&str> {
		self.0.resolve(symbol.0)
	}


	/// Get the number of interned strings.
	/// This does not include the dummy symbol.
	pub fn len(&self) -> usize {
		self.0.len() - 1
	}
}
