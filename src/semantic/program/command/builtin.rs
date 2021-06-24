use std::collections::HashSet;


pub fn is_builtin(command: &[u8]) -> bool {
	const BUILTINS: &'static [&'static [u8]] = &[
		b"cd",
		b"alias",
	];

	thread_local! {
		static BUILTINS_SET: HashSet<&'static [u8]> = BUILTINS.iter().cloned().collect();
	}

	BUILTINS_SET.with(|set| set.contains(command))
}
