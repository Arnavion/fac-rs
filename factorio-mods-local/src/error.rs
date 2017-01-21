/// Error kinds for errors returned by this crate.
#[derive(Debug, error_chain)]
pub enum ErrorKind {
	/// A generic error message
	Msg(String),

	/// An error encountered while iterating over a glob result
	#[error_chain(foreign)]
	Glob(::glob::GlobError),

	/// An IO error
	#[error_chain(custom)]
	#[error_chain(display = r#"(|f: &mut ::std::fmt::Formatter, path: &::std::path::Path, _| write!(f, "IO error on file {}", path.display()))"#)]
	#[error_chain(cause = "(|_, err| err)")]
	IO(::std::path::PathBuf, ::std::io::Error),

	/// Deserializing some JSON failed
	#[error_chain(custom)]
	#[error_chain(display = r#"(|f: &mut ::std::fmt::Formatter, path: &::std::path::Path, _| write!(f, "Could not parse the JSON file {}", path.display()))"#)]
	#[error_chain(cause = "(|_, err| err)")]
	JSON(::std::path::PathBuf, ::serde_json::Error),

	/// An error encountered while working with a .zip file
	#[error_chain(custom)]
	#[error_chain(display = r#"(|f: &mut ::std::fmt::Formatter, path: &::std::path::Path, _| write!(f, "Could not parse the ZIP file {}", path.display()))"#)]
	#[error_chain(cause = "(|_, err| err)")]
	Zip(::std::path::PathBuf, ::zip::result::ZipError),

	/// A zipped mod has no files and is thus malformed
	#[error_chain(custom)]
	#[error_chain(display = r#"(|f: &mut ::std::fmt::Formatter, path: &::std::path::Path| write!(f, "The zipped mod file {} is empty", path.display()))"#)]
	EmptyZippedMod(::std::path::PathBuf),

	/// The file or directory is not recognized as a valid mod format
	#[error_chain(custom)]
	#[error_chain(display = r#"(|f: &mut ::std::fmt::Formatter, path: &::std::path::Path| write!(f, "The mod at {} could not be recognized as a valid mod", path.display()))"#)]
	UnknownModFormat(::std::path::PathBuf),

	/// The glob cannot be constructed because the pattern is not UTF-8
	#[error_chain(custom)]
	#[error_chain(display = r#"(|f: &mut ::std::fmt::Formatter, pattern: &::std::ffi::OsStr| write!(f, "The pattern {:?} is not valid UTF-8 and thus cannot be converted into a glob", pattern))"#)]
	Utf8Path(::std::ffi::OsString),

	/// Generating a glob from a pattern failed
	#[error_chain(custom)]
	#[error_chain(display = r#"(|f: &mut ::std::fmt::Formatter, pattern, _| write!(f, "The pattern {} is invalid", pattern))"#)]
	#[error_chain(cause = "(|_, err| err)")]
	Pattern(String, ::glob::PatternError),

	/// The local Factorio installation could not be found.
	#[error_chain(custom)]
	#[error_chain(display = r#"(|f: &mut ::std::fmt::Formatter| write!(f, "The local Factorio installation could not be found"))"#)]
	DataPath,

	/// The local Factorio installation could not be found.
	#[error_chain(custom)]
	#[error_chain(display = r#"(|f: &mut ::std::fmt::Formatter| write!(f, "The local Factorio installation could not be found"))"#)]
	WritePath,

	/// The credentials stored in `player-data.json` do not have both username and service token.
	#[error_chain(custom)]
	#[error_chain(display = r#"(|f: &mut ::std::fmt::Formatter, _| write!(f, "Valid API credentials were not found in player-data.json"))"#)]
	IncompleteUserCredentials(Option<::factorio_mods_common::ServiceUsername>),
}
