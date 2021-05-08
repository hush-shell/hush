use super::*;

use assert_matches::assert_matches;


macro_rules! assert_token {
	($e:expr, $token_kind:pat) => {
		assert_matches!($e, Some(Token { kind: $token_kind, .. }))
	};

	($e:expr, $token_kind:pat => $arm:expr) => {
		assert_matches!($e, Some(Token { kind: $token_kind, .. }) => $arm)
	};
}


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

	let mut cursor = Cursor::new(input.as_bytes());
	let mut interner = symbol::Interner::new();

	macro_rules! next {
		() => { Lexer::read_token(&mut cursor, &mut interner) }
	}

	macro_rules! assert_symbol {
		($symbol:ident, $expected:literal) => {
			assert_eq!(
				interner.resolve($symbol),
				Some($expected)
			)
		}
	}

	assert_token!(next!(), TokenKind::Keyword(Keyword::Function));
	assert_token!(next!(), TokenKind::Identifier(symbol) => assert_symbol!(symbol, "foo"));
	assert_token!(next!(), TokenKind::OpenParens);
	assert_token!(next!(), TokenKind::Identifier(symbol) => assert_symbol!(symbol, "bar"));
	assert_token!(next!(), TokenKind::Comma);
	assert_token!(next!(), TokenKind::Identifier(symbol) => assert_symbol!(symbol, "baz"));
	assert_token!(next!(), TokenKind::CloseParens);
	assert_token!(next!(), TokenKind::Keyword(Keyword::If));
	assert_token!(next!(), TokenKind::Identifier(symbol) => assert_symbol!(symbol, "bar"));
	assert_token!(next!(), TokenKind::Operator(Operator::Or));
	assert_token!(next!(), TokenKind::Identifier(symbol) => assert_symbol!(symbol, "baz"));
	assert_token!(next!(), TokenKind::Operator(Operator::Equals));
	assert_token!(next!(), TokenKind::Literal(Literal::Nil));
	assert_token!(next!(), TokenKind::Keyword(Keyword::Then));
	assert_token!(next!(), TokenKind::Keyword(Keyword::Let));
	assert_token!(next!(), TokenKind::Identifier(symbol) => assert_symbol!(symbol, "result"));
	assert_token!(next!(), TokenKind::Operator(Operator::Assign));
	assert_token!(next!(), TokenKind::Identifier(symbol) => assert_symbol!(symbol, "do_something"));
	assert_token!(next!(), TokenKind::OpenParens);
	assert_token!(next!(), TokenKind::CloseParens);
	assert_token!(next!(), TokenKind::Keyword(Keyword::Return));
	assert_token!(next!(), TokenKind::Identifier(symbol) => assert_symbol!(symbol, "result"));
	assert_token!(next!(), TokenKind::Keyword(Keyword::End));
	assert_token!(next!(), TokenKind::Keyword(Keyword::End));
	assert_matches!(next!(), None);
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
	let tokens: Vec<TokenKind> = std::iter
		::from_fn(
			|| Lexer::read_token(&mut cursor, &mut interner)
		)
    .map(
			|token| token.kind
		)
    .collect();

	macro_rules! assert_symbol {
		($symbol:ident, $expected:literal) => {
			assert_eq!(
				interner.resolve(*$symbol),
				Some($expected)
			)
		}
	}

	assert_matches!(
		&tokens[..],
		[
			TokenKind::Keyword(Keyword::Function),
			TokenKind::Identifier(foo),
			TokenKind::OpenParens,
			TokenKind::Identifier(bar1),
			TokenKind::Comma,
			TokenKind::Identifier(baz1),
			TokenKind::CloseParens,
			TokenKind::Unexpected(b'|'),
			TokenKind::Keyword(Keyword::If),
			TokenKind::Identifier(bar2),
			TokenKind::Operator(Operator::Or),
			TokenKind::Identifier(baz2),
			TokenKind::Operator(Operator::Equals),
			TokenKind::Literal(Literal::Nil),
			TokenKind::Keyword(Keyword::Then),
			TokenKind::Keyword(Keyword::Let),
			TokenKind::Unexpected(b'$'),
			TokenKind::Identifier(result1),
			TokenKind::Operator(Operator::Assign),
			TokenKind::Identifier(do_something),
			TokenKind::OpenParens,
			TokenKind::CloseParens,
			TokenKind::Keyword(Keyword::Return),
			TokenKind::Unexpected(b'@'),
			TokenKind::Unexpected(b'}'),
			TokenKind::Identifier(result2),
			TokenKind::Keyword(Keyword::End),
			TokenKind::Keyword(Keyword::End),
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
