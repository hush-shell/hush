use super::{ast, TokenKind, Keyword};


/// A strategy for synchronizing the token stream.
#[derive(Debug)]
pub enum Strategy {
	/// Don't skip any token.
	Keep,
	/// Skip a single token.
	SkipOne {
		skipped: bool,
	},
	/// Skip until after a token is found.
	Token {
		token: TokenKind,
		found: bool,
	},
	/// Skip until after a keyword is found.
	Keyword {
		keyword: Keyword,
		found: bool,
	},
	/// Skip until after a block terminator token is matched.
	/// See TokenKind::is_block_terminator for more details.
	BlockTerminator {
		skipped: bool,
	},
	/// Skip until after a basic command terminator token is matched.
	/// See TokenKind::is_basic_command_terminator for more details.
	BasicCommandTerminator {
		skipped: bool,
	},
}


impl Strategy {
	/// A dummy strategy to use when EOF was already reached.
	/// The behavior of this strategy is irrelevant, because the parser can't skip beyond
	/// EOF.
	pub fn eof() -> Self {
		Self::Keep
	}


	/// Don't skip any token.
	pub fn keep() -> Self {
		Self::Keep
	}


	/// Skip a single token.
	pub fn skip_one() -> Self {
		Self::SkipOne { skipped: false }
	}


	/// Skip until after a token is found.
	pub fn token(token: TokenKind) -> Self {
		Self::Token { token, found: false }
	}


	pub fn keyword(keyword: Keyword) -> Self {
		Self::Keyword { keyword, found: false }
	}


	/// Skip until after a block terminator token is matched.
	/// See TokenKind::is_block_terminator for more details.
	pub fn block_terminator() -> Self {
		Self::BlockTerminator { skipped: false }
	}


	/// Skip until after a basic command terminator token is matched.
	/// See TokenKind::is_basic_command_terminator for more details.
	pub fn basic_command_terminator() -> Self {
		Self::BasicCommandTerminator { skipped: false }
	}


	/// Indicates whether the stream has been synchronized.
	/// When this method returns false, the token should be skipped.
	pub fn synchronized(&mut self, token: &TokenKind) -> bool {
		match self {
			Self::Keep => true,

			Self::SkipOne { skipped: true } => true,
			Self::SkipOne { skipped } => {
				*skipped = true;
				false
			},

			Self::Keyword { found: true, .. } => true,
			Self::Keyword { keyword, found } => {
				*found = matches!(token, TokenKind::Keyword(kwd) if kwd == keyword);
				false
			},

			Self::Token { found: true, .. } => true,
			Self::Token { token: expected, found } => {
				*found = token == expected;
				false
			},

			Self::BlockTerminator { skipped: true } => true,
			Self::BlockTerminator { skipped } => {
				*skipped = token.is_block_terminator();
				false
			},

			Self::BasicCommandTerminator { skipped: true } => true,
			Self::BasicCommandTerminator { skipped } => {
				*skipped = token.is_basic_command_terminator();
				false
			},
		}
	}
}


/// A parser that can be synchronized.
pub trait Synchronizable<E> {
	/// Synchronize using the given strategy.
	fn synchronize(&mut self, error: E, sync: Strategy);
}


/// A result inclusing a synchronization strategy.
pub type Result<T, E> = std::result::Result<T, (E, Strategy)>;


/// Extension trait for adding a sync strategy to a Result.
pub trait WithSync<T, E> {
	/// Use the given sync strategy.
	fn with_sync(self, strategy: Strategy) -> Result<T, E>;
}


impl<T, E> WithSync<T, E> for std::result::Result<T, E> {
	fn with_sync(self, strategy: Strategy) -> Result<T, E> {
		self.map_err(|error| (error, strategy))
	}
}


impl<T, E> WithSync<T, E> for Result<T, E> {
	fn with_sync(self, strategy: Strategy) -> Result<T, E> {
		self.map_err(|(error, _)| (error, strategy))
	}
}


/// Extension trait for synchronizing from Result.
pub trait ResultExt<T, E> {
	/// Synchronize the parser using the current strategy.
	fn synchronize<P: Synchronizable<E>>(self, parser: &mut P) -> T;

	/// If the sync strategy is `keep`, replace it with `skip_one`.
	fn force_sync_skip(self) -> Self;
}


impl<T, E> ResultExt<T, E> for Result<T, E>
where
	T: ast::IllFormed,
{
	fn synchronize<P: Synchronizable<E>>(self, parser: &mut P) -> T {
		match self {
			Ok(value) => value,

			Err((error, sync)) => {
				parser.synchronize(error, sync);
				T::ill_formed()
			}
		}
	}


	fn force_sync_skip(self) -> Self {
		self.map_err(
			|(error, strategy)| match strategy {
				Strategy::Keep => (error, Strategy::skip_one()),
				_ => (error, strategy)
			}
		)
	}
}
