mod fmt;

use intaglio::{Symbol as SymbolInner, bytes::SymbolTable};


/// A symbol is a reference to an value stored in the symbol interner.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Symbol(SymbolInner);


/// The default symbol is a dummy symbol, which will yield "<invalid symbol>" when
/// resolved.
impl Default for Symbol {
	fn default() -> Self {
		Self(SymbolInner::new(0))
	}
}


impl From<Symbol> for usize {
	fn from(symbol: Symbol) -> usize {
		symbol.0.id() as usize
	}
}


/// A symbol interner, used to store identifiers, paths, etc.
#[derive(Debug)]
pub struct Interner(SymbolTable);


impl Interner {
	/// Create a new interner. Please note that this allocates memory even if no symbols are
	/// inserted.
	pub fn new() -> Self {
		let mut interner = SymbolTable::new();
		interner
			.intern(b"<invalid symbol>".as_ref())
			.expect("failed to intern symbol");
		Self(interner)
	}


	/// Get the symbol for a value.
	#[cfg(test)]
	pub fn get<T>(&self, value: T) -> Option<Symbol>
	where
		T: AsRef<[u8]>,
	{
		self.0
			.check_interned(value.as_ref())
			.map(Symbol)
	}


	/// Get the symbol for a value. The value is interned if needed.
	pub fn get_or_intern<T>(&mut self, value: T) -> Symbol
	where
		T: AsRef<[u8]>,
	{
		let value = value.as_ref().to_owned();

		Symbol(
			self.0
				.intern(value)
				.expect("failed to intern symbol")
		)
	}


	/// Resolve the string for a symbol.
	pub fn resolve(&self, symbol: Symbol) -> Option<&[u8]> {
		self.0.get(symbol.0)
	}


	/// Get the number of interned strings.
	/// This does not include the dummy symbol.
	#[cfg(test)]
	pub fn len(&self) -> usize {
		self.0.len() - 1
	}
}
