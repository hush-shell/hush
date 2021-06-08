use super::*;

use assert_matches::assert_matches;


macro_rules! token {
	($kind:pat) => {
		Ok(Token { kind: $kind, .. })
	};
}

macro_rules! error {
	($error:pat) => {
		Err(Error { error: $error, .. })
	};
}

macro_rules! assert_symbol {
	($interner:ident, $symbol:ident, $expected:literal) => {
		assert_eq!($interner.resolve(*$symbol), Some($expected))
	};
}


/// Check that TokenKind is not too big, because it gets moved around a lot.
#[test]
fn test_token_kind_size() {
	assert_eq!(std::mem::size_of::<TokenKind>(), 32);
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

	let cursor = Cursor::from(input.as_bytes());
	let mut interner = symbol::Interner::new();
	let lexer = Lexer::new(cursor, &mut interner);

	let tokens: Vec<Result<Token, Error>> = lexer.collect();

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
			token!(TokenKind::Keyword(Keyword::If)),
			token!(TokenKind::Identifier(bar2)),
			token!(TokenKind::Operator(Operator::Or)),
			token!(TokenKind::Identifier(baz2)),
			token!(TokenKind::Operator(Operator::Equals)),
			token!(TokenKind::Literal(Literal::Nil)),
			token!(TokenKind::Keyword(Keyword::Then)),
			token!(TokenKind::Keyword(Keyword::Let)),
			token!(TokenKind::Identifier(result1)),
			token!(TokenKind::Operator(Operator::Assign)),
			token!(TokenKind::Identifier(do_something)),
			token!(TokenKind::OpenParens),
			token!(TokenKind::CloseParens),
			token!(TokenKind::Keyword(Keyword::Return)),
			token!(TokenKind::Identifier(result2)),
			token!(TokenKind::Keyword(Keyword::End)),
			token!(TokenKind::Keyword(Keyword::End)),
		]
			=> {
				assert_symbol!(interner, foo, "foo");
				assert_symbol!(interner, bar1, "bar");
				assert_symbol!(interner, bar2, "bar");
				assert_symbol!(interner, baz1, "baz");
				assert_symbol!(interner, baz2, "baz");
				assert_symbol!(interner, result1, "result");
				assert_symbol!(interner, result2, "result");
				assert_symbol!(interner, do_something, "do_something");
			}
	);
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

	let cursor = Cursor::from(input.as_bytes());
	let mut interner = symbol::Interner::new();
	let lexer = Lexer::new(cursor, &mut interner);

	let tokens: Vec<Result<Token, Error>> = lexer.collect();

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
			error!(ErrorKind::Unexpected(b'|')),
			token!(TokenKind::Keyword(Keyword::If)),
			token!(TokenKind::Identifier(bar2)),
			token!(TokenKind::Operator(Operator::Or)),
			token!(TokenKind::Identifier(baz2)),
			token!(TokenKind::Operator(Operator::Equals)),
			token!(TokenKind::Literal(Literal::Nil)),
			token!(TokenKind::Keyword(Keyword::Then)),
			token!(TokenKind::Keyword(Keyword::Let)),
			error!(ErrorKind::Unexpected(b'$')),
			token!(TokenKind::Identifier(result1)),
			token!(TokenKind::Operator(Operator::Assign)),
			token!(TokenKind::Identifier(do_something)),
			token!(TokenKind::OpenParens),
			token!(TokenKind::CloseParens),
			token!(TokenKind::Keyword(Keyword::Return)),
			error!(ErrorKind::Unexpected(b'@')),
			error!(ErrorKind::Unexpected(b'}')),
			token!(TokenKind::Identifier(result2)),
			token!(TokenKind::Keyword(Keyword::End)),
			token!(TokenKind::Keyword(Keyword::End)),
		]
			=> {
				assert_symbol!(interner, foo, "foo");
				assert_symbol!(interner, bar1, "bar");
				assert_symbol!(interner, bar2, "bar");
				assert_symbol!(interner, baz1, "baz");
				assert_symbol!(interner, baz2, "baz");
				assert_symbol!(interner, result1, "result");
				assert_symbol!(interner, result2, "result");
				assert_symbol!(interner, do_something, "do_something");
			}
	);
}


#[test]
fn test_byte_literals() {
	let input = r#"
		let var = 'a'
		var = '\n'
		var = '\?'   # invalid escape sequence
		var = '\na'  # invalid literal with escape sequence 1
		var = 'a\n'  # invalid literal with escape sequence 2
		var = '\1a'  # invalid escape sequence followed by character
	"#;

	let cursor = Cursor::from(input.as_bytes());
	let mut interner = symbol::Interner::new();
	let lexer = Lexer::new(cursor, &mut interner);

	let tokens: Vec<Result<Token, Error>> = lexer.collect();

	assert_matches!(
		&tokens[..],
		[
			token!(TokenKind::Keyword(Keyword::Let)),
			token!(TokenKind::Identifier(var)),
			token!(TokenKind::Operator(Operator::Assign)),
			token!(TokenKind::Literal(Literal::Byte(b'a'))),

			token!(TokenKind::Identifier(_)),
			token!(TokenKind::Operator(Operator::Assign)),
			token!(TokenKind::Literal(Literal::Byte(b'\n'))),

			token!(TokenKind::Identifier(_)),
			token!(TokenKind::Operator(Operator::Assign)),
			error!(ErrorKind::InvalidEscapeSequence(e1)),
			token!(TokenKind::Literal(Literal::Byte(_))),

			token!(TokenKind::Identifier(_)),
			token!(TokenKind::Operator(Operator::Assign)),
			error!(ErrorKind::Unexpected(b'a')),
			token!(TokenKind::Literal(Literal::Byte(b'\n'))),

			token!(TokenKind::Identifier(_)),
			token!(TokenKind::Operator(Operator::Assign)),
			error!(ErrorKind::Unexpected(b'\\')),
			error!(ErrorKind::Unexpected(b'n')),
			token!(TokenKind::Literal(Literal::Byte(b'a'))),

			token!(TokenKind::Identifier(_)),
			token!(TokenKind::Operator(Operator::Assign)),
			error!(ErrorKind::InvalidEscapeSequence(e2)),
			error!(ErrorKind::Unexpected(b'a')),
			token!(TokenKind::Literal(Literal::Byte(_))),
		]
			=> {
				assert_symbol!(interner, var, "var");
				assert_eq!(interner.len(), 1);
				assert_eq!(e1.as_ref(), b"\\?");
				assert_eq!(e2.as_ref(), b"\\1");
			}
	);
}


#[test]
fn test_string_literals() {
	let input = r#"
		let var = "hello world" ++ "escape \n sequences \" are \0 cool" ++ ""
	"#;

	let cursor = Cursor::from(input.as_bytes());
	let mut interner = symbol::Interner::new();
	let lexer = Lexer::new(cursor, &mut interner);

	let tokens: Vec<Result<Token, Error>> = lexer.collect();

	assert_matches!(
		&tokens[..],
		[
			token!(TokenKind::Keyword(Keyword::Let)),
			token!(TokenKind::Identifier(var)),
			token!(TokenKind::Operator(Operator::Assign)),
			token!(TokenKind::Literal(Literal::String(lit1))),
			token!(TokenKind::Operator(Operator::Concat)),
			token!(TokenKind::Literal(Literal::String(lit2))),
			token!(TokenKind::Operator(Operator::Concat)),
			token!(TokenKind::Literal(Literal::String(lit3))),
		]
			=> {
				assert_symbol!(interner, var, "var");
				assert_eq!(lit1.as_ref(), b"hello world");
				assert_eq!(lit2.as_ref(), b"escape \n sequences \" are \0 cool");
				assert!(lit3.is_empty());
			}
	);
}


#[test]
fn test_number_literals() {
	let input = r#"
		let var = 123 + 456.7 + 89e10 + 1.2e3
	"#;

	let cursor = Cursor::from(input.as_bytes());
	let mut interner = symbol::Interner::new();
	let lexer = Lexer::new(cursor, &mut interner);

	let tokens: Vec<Result<Token, Error>> = lexer.collect();

	assert_matches!(
		&tokens[..],
		[
			token!(TokenKind::Keyword(Keyword::Let)),
			token!(TokenKind::Identifier(var)),
			token!(TokenKind::Operator(Operator::Assign)),
			token!(TokenKind::Literal(Literal::Int(i1))),
			token!(TokenKind::Operator(Operator::Plus)),
			token!(TokenKind::Literal(Literal::Float(f1))),
			token!(TokenKind::Operator(Operator::Plus)),
			token!(TokenKind::Literal(Literal::Float(f2))),
			token!(TokenKind::Operator(Operator::Plus)),
			token!(TokenKind::Literal(Literal::Float(f3))),
		]
			=> {
				assert_symbol!(interner, var, "var");
				assert_eq!(*i1, 123);
				assert_eq!(*f1, 456.7);
				assert_eq!(*f2, 89e10);
				assert_eq!(*f3, 1.2e3);
			}
	);
}


#[test]
fn test_command_block() {
	let input = r#"
		let result = {
			here-is-some 1arg "2arg" '3arg' 4'arg' >> "5"arg?;
			$dollars ${are} "$fun";
			so\ are escape \> \< \; sequences \?
		}
	"#;

	let cursor = Cursor::from(input.as_bytes());
	let mut interner = symbol::Interner::new();
	let lexer = Lexer::new(cursor, &mut interner);

	let tokens: Vec<Result<Token, Error>> = lexer.collect();

	let unquoted = ArgPart::Unquoted;
	let single_quoted = |arg: &str| ArgPart::SingleQuoted(arg.as_bytes().into());
	let double_quoted = |parts: &[ArgUnit]| ArgPart::DoubleQuoted(parts.to_vec().into());

	let literal = |lit: &str| ArgUnit::Literal(lit.as_bytes().into());
	let dollar = |ident: &str| ArgUnit::Dollar(interner.get(ident).expect("symbol not found"));

	assert_matches!(
		&tokens[..],
		[
			token!(TokenKind::Keyword(Keyword::Let)),
			token!(TokenKind::Identifier(var)),
			token!(TokenKind::Operator(Operator::Assign)),
			token!(TokenKind::Command),
			token!(TokenKind::Argument(args0)),
			token!(TokenKind::Argument(args1)),
			token!(TokenKind::Argument(args2)),
			token!(TokenKind::Argument(args3)),
			token!(TokenKind::Argument(args4)),
			token!(TokenKind::CmdOperator(CommandOperator::Output { append: true })),
			token!(TokenKind::Argument(args5)),
			token!(TokenKind::CmdOperator(CommandOperator::Try)),
			token!(TokenKind::Semicolon),
			token!(TokenKind::Argument(dollar1)),
			token!(TokenKind::Argument(dollar2)),
			token!(TokenKind::Argument(dollar3)),
			token!(TokenKind::Semicolon),
			token!(TokenKind::Argument(args6)),
			token!(TokenKind::Argument(args7)),
			token!(TokenKind::Argument(gt)),
			token!(TokenKind::Argument(lt)),
			token!(TokenKind::Argument(semicolon)),
			token!(TokenKind::Argument(args8)),
			token!(TokenKind::Argument(question)),
			token!(TokenKind::CloseCommand),
		]
			=> {
				assert_symbol!(interner, var, "result");

				assert_eq!(args0.as_ref(), &[unquoted(literal("here-is-some"))]);
				assert_eq!(args1.as_ref(), &[unquoted(literal("1arg"))]);
				assert_eq!(args2.as_ref(), &[double_quoted(&[literal("2arg")])]);
				assert_eq!(args3.as_ref(), &[single_quoted("3arg")]);
				assert_eq!(args4.as_ref(), &[unquoted(literal("4")), single_quoted("arg")]);
				assert_eq!(args5.as_ref(), &[double_quoted(&[literal("5")]), unquoted(literal("arg"))]);

				assert_eq!(dollar1.as_ref(), &[unquoted(dollar("dollars"))]);
				assert_eq!(dollar2.as_ref(), &[unquoted(dollar("are"))]);
				assert_eq!(dollar3.as_ref(), &[double_quoted(&[dollar("fun")])]);

				assert_eq!(args6.as_ref(), &[unquoted(literal("so are"))]);
				assert_eq!(args7.as_ref(), &[unquoted(literal("escape"))]);
				assert_eq!(gt.as_ref(), &[unquoted(literal(">"))]);
				assert_eq!(lt.as_ref(), &[unquoted(literal("<"))]);
				assert_eq!(semicolon.as_ref(), &[unquoted(literal(";"))]);
				assert_eq!(args8.as_ref(), &[unquoted(literal("sequences"))]);
				assert_eq!(question.as_ref(), &[unquoted(literal("?"))]);
			}
	);
}
