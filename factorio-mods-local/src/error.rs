// Error type for errors returned by this crate.
error_chain! {
	foreign_links {
		// An error encountered while iterating over a glob result
		::glob::GlobError, Glob;

		// An IO error
		::std::io::Error, IO;

		// Deserializing some JSON failed
		::serde_json::Error, Json;

		// Generating a glob from a pattern failed
		::glob::PatternError, Pattern;

		// An error encountered while working with a .zip file
		::zip::result::ZipError, Zip;
	}

	errors {
		// A zipped mod has no files and is thus malformed
		EmptyZippedMod(path: ::std::path::PathBuf) { }

		// The file or directory is not recognized as a valid mod format
		UnknownModFormat { }

		// The glob cannot be constructed because the pattern is not UTF-8
		Utf8Path(glob_pattern: ::std::path::PathBuf) { }

		// The local Factorio installation could not be found.
		WritePath { }

		// The credentials stored in `player-data.json` do not have both username and service token.
		IncompleteUserCredentials(username: Option<::factorio_mods_common::ServiceUsername>) { }
	}
}
