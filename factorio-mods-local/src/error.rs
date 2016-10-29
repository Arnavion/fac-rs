error_chain! {
	foreign_links {
		::glob::GlobError, Glob;
		::std::io::Error, IO;
		::serde_json::Error, Json;
		::glob::PatternError, Pattern;
		::zip::result::ZipError, Zip;
	}

	errors {
		EmptyZippedMod(path: ::std::path::PathBuf) { }
		UnknownModFormat { }
		Utf8Path(glob_pattern: ::std::path::PathBuf) { }
		WritePath { }
		IncompleteUserCredentials(username: Option<::factorio_mods_common::ServiceUsername>) { }
	}
}
