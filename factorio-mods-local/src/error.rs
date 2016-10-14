error_chain! {
	foreign_links {
		::glob::PatternError, Pattern;
		::glob::GlobError, Glob;
		::std::io::Error, ModListFile;
		::serde_json::Error, ModListJson;
		::zip::result::ZipError, BadZippedMod;
	}

	errors {
		Utf8Path(glob_pattern: ::std::path::PathBuf) { }
		WritePath { }
		ModList(expected_path: ::std::path::PathBuf) { }
		EmptyZippedMod(path: ::std::path::PathBuf) { }
		UnknownModFormat { }
	}
}
