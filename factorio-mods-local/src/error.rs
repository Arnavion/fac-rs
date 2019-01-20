/// Errors returned by this crate.
#[derive(Debug)]
pub struct Error {
	kind: ErrorKind,
	backtrace: failure::Backtrace,
}

impl Error {
	/// Gets the kind of error
	pub fn kind(&self) -> &ErrorKind {
		&self.kind
	}
}

impl failure::Fail for Error {
	fn cause(&self) -> Option<&dyn failure::Fail> {
		self.kind.cause()
	}

	fn backtrace(&self) -> Option<&failure::Backtrace> {
		Some(&self.backtrace)
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.kind.fmt(f)
	}
}

impl From<ErrorKind> for Error {
	fn from(kind: ErrorKind) -> Self {
		Error {
			kind,
			backtrace: Default::default(),
		}
	}
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		ErrorKind::IO(err).into()
	}
}

/// Error kinds for errors returned by this crate.
#[derive(Debug, failure_derive::Fail)]
pub enum ErrorKind {
	/// An IO error
	#[fail(display = "IO error")]
	IO(#[cause] std::io::Error),

	/// An IO error
	#[fail(display = "IO error on file {}", _0)]
	FileIO(DisplayablePathBuf, #[cause] std::io::Error),

	/// Reading a JSON file failed
	#[fail(display = "Could not parse the JSON file {}", _0)]
	ReadJSONFile(DisplayablePathBuf, #[cause] serde_json::Error),

	/// Writing a JSON file failed
	#[fail(display = "Could not save {}", _0)]
	WriteJSONFile(DisplayablePathBuf, #[cause] serde_json::Error),

	/// An error encountered while working with a .zip file
	#[fail(display = "Could not parse the ZIP file {}", _0)]
	Zip(DisplayablePathBuf, #[cause] zip::result::ZipError),

	/// A zipped mod has no files and is thus malformed
	#[fail(display = "The zipped mod file {} is empty", _0)]
	EmptyZippedMod(DisplayablePathBuf),

	/// The file or directory is not recognized as a valid mod format
	#[fail(display = "The mod at {} could not be recognized as a valid mod", _0)]
	UnknownModFormat(DisplayablePathBuf),

	/// Generating a glob from a pattern failed
	#[fail(display = "The pattern {} is invalid", _0)]
	Pattern(String, #[cause] globset::Error),

	/// The local Factorio installation could not be found.
	#[fail(display = "The local Factorio installation could not be found")]
	DataPath,

	/// The local Factorio installation could not be found.
	#[fail(display = "The local Factorio installation could not be found")]
	WritePath,

	/// The credentials stored in `player-data.json` do not have both username and service token.
	#[fail(display = "Valid API credentials were not found in player-data.json")]
	IncompleteUserCredentials(Option<factorio_mods_common::ServiceUsername>),
}

#[derive(Debug)]
pub struct DisplayablePathBuf(pub std::path::PathBuf);

impl std::fmt::Display for DisplayablePathBuf {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0.display())
	}
}

impl<P> From<P> for DisplayablePathBuf where P: Into<std::path::PathBuf> {
	fn from(path: P) -> Self {
		DisplayablePathBuf(path.into())
	}
}
