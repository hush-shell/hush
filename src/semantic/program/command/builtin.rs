use std::collections::HashSet;

use lazy_static::lazy_static;


const BUILTINS: &'static [&'static [u8]] = &[
	b"cd",
	b"alias",
];


pub fn is_builtin(command: &[u8]) -> bool {
	lazy_static! {
		static ref BUILTINS_SET: HashSet<&'static [u8]> = {
			let mut set = HashSet::new();
			set.extend(BUILTINS);
			set
		};
	}

	BUILTINS_SET.contains(command)
}
