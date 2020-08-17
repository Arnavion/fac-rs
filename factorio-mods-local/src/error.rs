/// Errors returned by this crate.
#[derive(Debug)]
pub enum Error {
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

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::EmptyZippedMod(path) => write!(f, "the zipped mod file {} is empty", path.display()),
			Error::IncompleteUserCredentials(_) => f.write_str("valid API credentials were not found in player-data.json"),
			Error::InstallDirectoryNotFound => f.write_str("the local Factorio installation could not be found"),
			Error::Io(path, _) => write!(f, "I/O error on {}", path.display()),
			Error::Pattern(pattern, _) => write!(f, "the pattern {} is invalid", pattern),
			Error::ReadJSONFile(path, _) => write!(f, "could not parse the JSON file {}", path.display()),
			Error::UnknownModFormat(path) => write!(f, "the mod at {} could not be recognized as a valid mod", path.display()),
			Error::UserDirectoryNotFound => f.write_str("the Factorio user directory could not be found"),
			Error::WriteJSONFile(path, _) => write!(f, "could not save {}", path.display()),
			Error::Zip(path, _) => write!(f, "could not parse the ZIP file {}", path.display()),
		}
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		#[allow(clippy::match_same_arms)]
		match self {
			Error::EmptyZippedMod(_) => None,
			Error::IncompleteUserCredentials(_) => None,
			Error::InstallDirectoryNotFound => None,
			Error::Io(_, err) => Some(err),
			Error::Pattern(_, err) => Some(err),
			Error::ReadJSONFile(_, err) => Some(err),
			Error::UnknownModFormat(_) => None,
			Error::UserDirectoryNotFound => None,
			Error::WriteJSONFile(_, err) => Some(err),
			Error::Zip(_, err) => Some(err),
		}
	}
}
