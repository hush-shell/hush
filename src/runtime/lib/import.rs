use std::{
	io,
	path::{Path, PathBuf},
	ffi::OsStr,
	os::unix::ffi::OsStrExt,
};

use gc::{Finalize, Trace};

use crate::{
	fmt,
	syntax,
	semantic,
	symbol::{self, Symbol}
};
use super::{
	CallContext,
	RustFun,
	NativeFun,
	Panic,
	Value,
};


inventory::submit! { RustFun::from(Import) }

#[derive(Trace, Finalize)]
struct Import;

impl Import {
	fn import(module_path: &Path, mut context: CallContext) -> Result<Value, Panic> {
		let path = Self
			::resolve_path(
				module_path,
				context.pos.path,
				context.runtime.interner_mut()
			)
			.map_err(
				|error| Panic::io(error, context.pos.copy())
			)?;

		match context.runtime.modules.get(&path) {
			Some(module) => Ok(module.copy()), // Don't reload module if cached.
			None => {
				let module = Self::load(path, &mut context)?;
				context.runtime.modules.insert(path, module.copy());
				Ok(module)
			}
		}
	}


	fn resolve_path(
		target_path: &Path,
		current_path: Symbol,
		interner: &mut symbol::Interner,
	) -> io::Result<Symbol> {
		let mut path_buf = PathBuf::from(
			OsStr::from_bytes(
				interner
					.resolve(current_path)
					.expect("failed to resolve symbol")
			).to_owned()
		);
		path_buf.pop(); // Remove the file name.
		path_buf.push(target_path);

		let path = path_buf.canonicalize()?;

		let path_symbol = interner.get_or_intern(
			path
				.as_os_str()
				.as_bytes()
		);

		Ok(path_symbol)
	}


	fn load(path: Symbol, context: &mut CallContext) -> Result<Value, Panic> {
		// Load file.
		let source = syntax::Source
			::from_path(
				path,
				context.runtime.interner_mut()
			).map_err(
				|error| Panic::io(error, context.pos.copy())
			)?;

		// Syntax.
		let syntactic_analysis = syntax::Analysis::analyze(
			source,
			context.runtime.interner_mut()
		);
		let has_syntax_errors = !syntactic_analysis.is_ok();

		if has_syntax_errors {
			eprint!("{}", fmt::Show(
				syntactic_analysis.errors,
				syntax::AnalysisDisplayContext {
					max_errors: Some(20),
					interner: context.runtime.interner(),
				}
			));
			return Err(Panic::import_failed(path, context.pos.copy()));
		}

		// Semantics.
		let program = semantic::Analyzer
			::analyze(
				syntactic_analysis.ast, context.runtime.interner_mut()
			)
			.map_err(
				|errors| {
					eprint!("{}", fmt::Show(
						errors,
						semantic::ErrorsDisplayContext {
							max_errors: Some(20),
							interner: context.runtime.interner(),
						}
					));

					Panic::import_failed(path, context.pos.copy())
				}
			)?;

		// Eval.
		let program = Box::leak(Box::new(program));
		context.runtime.eval(program)
	}
}

impl NativeFun for Import {
	fn name(&self) -> &'static str { "std.import" }

	fn call(&self, context: CallContext) -> Result<Value, Panic> {
		let path = match context.args() {
			[ Value::String(ref string) ] => Path::new(string).to_owned(),

			[ other ] => return Err(Panic::type_error(other.copy(), "string", context.pos)),

			args => return Err(Panic::invalid_args(args.len() as u32, 1, context.pos))
		};

		Self::import(&path, context)
	}
}
