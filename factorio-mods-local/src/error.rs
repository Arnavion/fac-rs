/// Error kinds for errors returned by this crate.
#[derive(Debug, error_chain)]
pub enum ErrorKind {
	/// A generic error message
	Msg(String),

	/// An error encountered while iterating over a glob result
	#[error_chain(foreign)]
	Glob(::glob::GlobError),

	/// An IO error
	#[error_chain(foreign)]
	IO(::std::io::Error),

	/// Deserializing some JSON failed
	#[error_chain(foreign)]
	JSON(::serde_json::Error),

	/// Generating a glob from a pattern failed
	#[error_chain(foreign)]
	Pattern(::glob::PatternError),

	/// An error encountered while working with a .zip file
	#[error_chain(foreign)]
	Zip(::zip::result::ZipError),

	/// A zipped mod has no files and is thus malformed
	#[error_chain(custom)]
	EmptyZippedMod(::std::path::PathBuf),

	/// The file or directory is not recognized as a valid mod format
	#[error_chain(custom)]
	UnknownModFormat,

	/// The glob cannot be constructed because the pattern is not UTF-8
	#[error_chain(custom)]
	Utf8Path(::std::path::PathBuf),

	/// The local Factorio installation could not be found.
	#[error_chain(custom)]
	DataPath,

	/// The local Factorio installation could not be found.
	#[error_chain(custom)]
	WritePath,

	/// The credentials stored in `player-data.json` do not have both username and service token.
	#[error_chain(custom)]
	IncompleteUserCredentials(Option<::factorio_mods_common::ServiceUsername>),
}
