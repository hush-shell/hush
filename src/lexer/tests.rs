use super::*;

use assert_matches::assert_matches;


#[test]
fn test_simple_function() {
	let input = r#"
		function foo(bar, baz)
			if bar or baz == nil then # here's a comment
				let result = do_something()
				return result
			end
		end
	"#;

	let cursor = Cursor::new(input.as_bytes());
	let mut interner = SymbolInterner::new();
	let mut lexer = Lexer::new(cursor, &mut interner);

	macro_rules! assert_token {
		($token_kind:pat) => {
			assert_matches!(lexer.next(), Some(Ok(Token { token: $token_kind, .. })))
		};

		($token_kind:pat => $arm:expr) => {
			assert_matches!(lexer.next(), Some(Ok(Token { token: $token_kind, .. })) => $arm)
		};
	}

	macro_rules! assert_symbol {
		($symbol:ident, $expected:literal) => {
			assert_eq!(interner.resolve($symbol), Some($expected))
		};
	}

	assert_token!(TokenKind::Keyword(Keyword::Function));
	assert_token!(TokenKind::Identifier(symbol) => assert_symbol!(symbol, "foo"));
	assert_token!(TokenKind::OpenParens);
	assert_token!(TokenKind::Identifier(symbol) => assert_symbol!(symbol, "bar"));
	assert_token!(TokenKind::Comma);
	assert_token!(TokenKind::Identifier(symbol) => assert_symbol!(symbol, "baz"));
	assert_token!(TokenKind::CloseParens);
	assert_token!(TokenKind::Keyword(Keyword::If));
	assert_token!(TokenKind::Identifier(symbol) => assert_symbol!(symbol, "bar"));
	assert_token!(TokenKind::Operator(Operator::Or));
	assert_token!(TokenKind::Identifier(symbol) => assert_symbol!(symbol, "baz"));
	assert_token!(TokenKind::Operator(Operator::Equals));
	assert_token!(TokenKind::Literal(Literal::Nil));
	assert_token!(TokenKind::Keyword(Keyword::Then));
	assert_token!(TokenKind::Keyword(Keyword::Let));
	assert_token!(TokenKind::Identifier(symbol) => assert_symbol!(symbol, "result"));
	assert_token!(TokenKind::Operator(Operator::Assign));
	assert_token!(TokenKind::Identifier(symbol) => assert_symbol!(symbol, "do_something"));
	assert_token!(TokenKind::OpenParens);
	assert_token!(TokenKind::CloseParens);
	assert_token!(TokenKind::Keyword(Keyword::Return));
	assert_token!(TokenKind::Identifier(symbol) => assert_symbol!(symbol, "result"));
	assert_token!(TokenKind::Keyword(Keyword::End));
	assert_token!(TokenKind::Keyword(Keyword::End));
	assert_matches!(lexer.next(), None);
}


#[test]
fn test_invalid_tokens() {
	let input = r#"
		function foo(bar, baz) |
			if bar or baz == nil then # here's a comment
				let $result = do_something()
				return @}result
			end
		end
	"#;

	let mut cursor = Cursor::new(input.as_bytes());
	let mut interner = symbol::Interner::new();
	let tokens: Vec<Token<TokenKind, Error<'_>>> = std::iter
		::from_fn(
			|| {
				match Lexer::read_token(&mut cursor, &mut interner) {
					Token::Eof => None,
					other => Some(other),
				}
			}
		)
		.collect();

	macro_rules! assert_symbol {
		($symbol:ident, $expected:literal) => {
			assert_eq!(interner.resolve(*$symbol), Some($expected))
		};
	}

	macro_rules! token {
		($kind:pat) => {
			Token::Token { kind: $kind, .. }
		};
	}

	macro_rules! error {
		($error:pat) => {
			Token::Error { error: $error, .. }
		};
	}

	assert_matches!(
		&tokens[..],
		[
			token!(TokenKind::Keyword(Keyword::Function)),
			token!(TokenKind::Identifier(foo)),
			token!(TokenKind::OpenParens),
			token!(TokenKind::Identifier(bar1)),
			token!(TokenKind::Comma),
			token!(TokenKind::Identifier(baz1)),
			token!(TokenKind::CloseParens),
			error!(Error::Unexpected(b'|')),
			token!(TokenKind::Keyword(Keyword::If)),
			token!(TokenKind::Identifier(bar2)),
			token!(TokenKind::Operator(Operator::Or)),
			token!(TokenKind::Identifier(baz2)),
			token!(TokenKind::Operator(Operator::Equals)),
			token!(TokenKind::Literal(Literal::Nil)),
			token!(TokenKind::Keyword(Keyword::Then)),
			token!(TokenKind::Keyword(Keyword::Let)),
			error!(Error::Unexpected(b'$')),
			token!(TokenKind::Identifier(result1)),
			token!(TokenKind::Operator(Operator::Assign)),
			token!(TokenKind::Identifier(do_something)),
			token!(TokenKind::OpenParens),
			token!(TokenKind::CloseParens),
			token!(TokenKind::Keyword(Keyword::Return)),
			error!(Error::Unexpected(b'@')),
			error!(Error::Unexpected(b'}')),
			token!(TokenKind::Identifier(result2)),
			token!(TokenKind::Keyword(Keyword::End)),
			token!(TokenKind::Keyword(Keyword::End)),
		]
			=> {
				assert_symbol!(foo, "foo");
				assert_symbol!(bar1, "bar");
				assert_symbol!(bar2, "bar");
				assert_symbol!(baz1, "baz");
				assert_symbol!(baz2, "baz");
				assert_symbol!(result1, "result");
				assert_symbol!(result2, "result");
				assert_symbol!(do_something, "do_something");
			}
	);
}


#[test]
fn test_string_literals() {
	let input = r#"
		let var = "hello world" ++ "escape \n sequences \" are \0 cool"
	"#;

	let mut cursor = Cursor::new(input.as_bytes());
	let mut interner = symbol::Interner::new();
	let tokens: Vec<Token<TokenKind, Error<'_>>> = std::iter
		::from_fn(
			|| {
				match Lexer::read_token(&mut cursor, &mut interner) {
					Token::Eof => None,
					other => Some(other),
				}
			}
		)
		.collect();

	macro_rules! assert_symbol {
		($symbol:ident, $expected:literal) => {
			assert_eq!(interner.resolve(*$symbol), Some($expected))
		};
	}

	macro_rules! token {
		($kind:pat) => {
			Token::Token { kind: $kind, .. }
		};
	}

	println!("{:#?}", tokens);

	assert_matches!(
		&tokens[..],
		[
			token!(TokenKind::Keyword(Keyword::Let)),
			token!(TokenKind::Identifier(var)),
			token!(TokenKind::Operator(Operator::Assign)),
			token!(TokenKind::Literal(Literal::String(lit1))),
			token!(TokenKind::Operator(Operator::Concat)),
			token!(TokenKind::Literal(Literal::String(lit2))),
		]
			=> {
				assert_symbol!(var, "var");
				assert_matches!(lit1.as_ref(), b"hello world");
				assert_matches!(lit2.as_ref(), b"escape \n sequences \" are \0 cool");
			}
	);
}
