/// Errors returned by this crate.
#[derive(Debug)]
pub struct Error {
	/// The kind of the error.
	pub kind: ErrorKind,

	/// The backtrace of the error.
	pub backtrace: backtrace::Backtrace,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "{}", self.kind)?;
		writeln!(f)?;
		writeln!(f, "{:?}", self.backtrace)?;
		Ok(())
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		self.kind.source()
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

/// Error kinds for errors returned by this crate.
#[derive(Debug)]
pub enum ErrorKind {
	/// A zipped mod has no files and is thus malformed.
	EmptyZippedMod(std::path::PathBuf),

	/// The credentials stored in `player-data.json` do not have both username and service token.
	IncompleteUserCredentials(Option<factorio_mods_common::ServiceUsername>),

	/// The local Factorio installation could not be found.
	InstallDirectoryNotFound,

	/// An I/O error.
	Io(std::path::PathBuf, std::io::Error),

	/// Generating a glob from a pattern failed.
	Pattern(String, globset::Error),

	/// Reading a JSON file failed.
	ReadJSONFile(std::path::PathBuf, serde_json::Error),

	/// The file or directory is not recognized as a valid mod format.
	UnknownModFormat(std::path::PathBuf),

	/// The Factorio user directory could not be found.
	UserDirectoryNotFound,

	/// Writing a JSON file failed.
	WriteJSONFile(std::path::PathBuf, serde_json::Error),

	/// An error encountered while working with a .zip file.
	Zip(std::path::PathBuf, zip::result::ZipError),
}

impl std::fmt::Display for ErrorKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ErrorKind::EmptyZippedMod(path) => write!(f, "the zipped mod file {} is empty", path.display()),
			ErrorKind::IncompleteUserCredentials(_) => f.write_str("valid API credentials were not found in player-data.json"),
			ErrorKind::InstallDirectoryNotFound => f.write_str("the local Factorio installation could not be found"),
			ErrorKind::Io(path, _) => write!(f, "I/O error on {}", path.display()),
			ErrorKind::Pattern(pattern, _) => write!(f, "the pattern {} is invalid", pattern),
			ErrorKind::ReadJSONFile(path, _) => write!(f, "could not parse the JSON file {}", path.display()),
			ErrorKind::UnknownModFormat(path) => write!(f, "the mod at {} could not be recognized as a valid mod", path.display()),
			ErrorKind::UserDirectoryNotFound => f.write_str("the Factorio user directory could not be found"),
			ErrorKind::WriteJSONFile(path, _) => write!(f, "could not save {}", path.display()),
			ErrorKind::Zip(path, _) => write!(f, "could not parse the ZIP file {}", path.display()),
		}
	}
}

impl std::error::Error for ErrorKind {
	#[allow(clippy::match_same_arms)]
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			ErrorKind::EmptyZippedMod(_) => None,
			ErrorKind::IncompleteUserCredentials(_) => None,
			ErrorKind::InstallDirectoryNotFound => None,
			ErrorKind::Io(_, err) => Some(err),
			ErrorKind::Pattern(_, err) => Some(err),
			ErrorKind::ReadJSONFile(_, err) => Some(err),
			ErrorKind::UnknownModFormat(_) => None,
			ErrorKind::UserDirectoryNotFound => None,
			ErrorKind::WriteJSONFile(_, err) => Some(err),
			ErrorKind::Zip(_, err) => Some(err),
		}
	}
}
