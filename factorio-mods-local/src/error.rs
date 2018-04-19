/// Error kinds for errors returned by this crate.
#[derive(Debug, ::derive_error_chain::ErrorChain)]
pub enum ErrorKind {
	/// An IO error
	#[error_chain(foreign)]
	IO(::std::io::Error),

	/// An IO error
	#[error_chain(custom)]
	#[error_chain(display = |path: &::std::path::Path, _| write!(f, "IO error on file {}", path.display()))]
	#[error_chain(cause = |_, err| err)]
	FileIO(::std::path::PathBuf, ::std::io::Error),

	/// Reading a JSON file failed
	#[error_chain(custom)]
	#[error_chain(display = |path: &::std::path::Path, _| write!(f, "Could not parse the JSON file {}", path.display()))]
	#[error_chain(cause = |_, err| err)]
	ReadJSONFile(::std::path::PathBuf, ::serde_json::Error),

	/// Writing a JSON file failed
	#[error_chain(custom)]
	#[error_chain(display = |path: &::std::path::Path, _| write!(f, "Could not save {}", path.display()))]
	#[error_chain(cause = |_, err| err)]
	WriteJSONFile(::std::path::PathBuf, ::serde_json::Error),

	/// An error encountered while working with a .zip file
	#[error_chain(custom)]
	#[error_chain(display = |path: &::std::path::Path, _| write!(f, "Could not parse the ZIP file {}", path.display()))]
	#[error_chain(cause = |_, err| err)]
	Zip(::std::path::PathBuf, ::zip::result::ZipError),

	/// A zipped mod has no files and is thus malformed
	#[error_chain(custom)]
	#[error_chain(display = |path: &::std::path::Path| write!(f, "The zipped mod file {} is empty", path.display()))]
	EmptyZippedMod(::std::path::PathBuf),

	/// The file or directory is not recognized as a valid mod format
	#[error_chain(custom)]
	#[error_chain(display = |path: &::std::path::Path| write!(f, "The mod at {} could not be recognized as a valid mod", path.display()))]
	UnknownModFormat(::std::path::PathBuf),

	/// Generating a glob from a pattern failed
	#[error_chain(custom)]
	#[error_chain(display = const("The pattern {0} is invalid"))]
	#[error_chain(cause = |_, err| err)]
	Pattern(String, ::globset::Error),

	/// The local Factorio installation could not be found.
	#[error_chain(custom)]
	#[error_chain(display = const("The local Factorio installation could not be found"))]
	DataPath,

	/// The local Factorio installation could not be found.
	#[error_chain(custom)]
	#[error_chain(display = const("The local Factorio installation could not be found"))]
	WritePath,

	/// The credentials stored in `player-data.json` do not have both username and service token.
	#[error_chain(custom)]
	#[error_chain(display = const("Valid API credentials were not found in player-data.json"))]
	IncompleteUserCredentials(Option<::factorio_mods_common::ServiceUsername>),
}
