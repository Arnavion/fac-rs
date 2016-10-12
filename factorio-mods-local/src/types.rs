#[derive(Debug)]
pub enum LocalError {
	Utf8Path { path: ::std::path::PathBuf, backtrace: Option<::backtrace::Backtrace> },
	Pattern { error: ::glob::PatternError, backtrace: Option<::backtrace::Backtrace> },
	Glob { error: ::glob::GlobError, backtrace: Option<::backtrace::Backtrace> },
	WritePath { backtrace: Option<::backtrace::Backtrace> },
	Other { backtrace: Option<::backtrace::Backtrace> },
}

impl LocalError {
	pub fn utf8_path(path: ::std::path::PathBuf) -> LocalError {
		LocalError::Utf8Path { path: path, backtrace: LocalError::backtrace() }
	}

	pub fn pattern(error: ::glob::PatternError) -> LocalError {
		LocalError::Pattern { error: error, backtrace: LocalError::backtrace() }
	}

	pub fn glob(error: ::glob::GlobError) -> LocalError {
		LocalError::Glob { error: error, backtrace: LocalError::backtrace() }
	}

	pub fn write_path() -> LocalError {
		LocalError::WritePath { backtrace: LocalError::backtrace() }
	}

	pub fn other() -> LocalError {
		LocalError::Other { backtrace: LocalError::backtrace() }
	}

	fn backtrace() -> Option<::backtrace::Backtrace> {
		::std::env::var("RUST_BACKTRACE").ok()
			.and_then(|value| { if value == "1" { Some(::backtrace::Backtrace::new()) } else { None } })
	}
}
