pub use string_interner::Symbol as SymbolExt;
use string_interner::{DefaultSymbol, StringInterner};


/// A symbol is a reference to an identifier stored in the string interner.
pub type Symbol = DefaultSymbol;


/// A string interner, used to store identifiers.
pub type Interner = StringInterner;
