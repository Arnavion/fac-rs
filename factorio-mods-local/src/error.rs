error_chain! {
	foreign_links {
		::glob::PatternError, Pattern;
		::glob::GlobError, Glob;
		::std::io::Error, ModListFile;
		::serde_json::Error, ModListJson;
	}

	errors {
		Utf8Path(glob_pattern: ::std::path::PathBuf) { }
		WritePath { }
		ModList(expected_path: ::std::path::PathBuf) { }
		BadZippedMod(path: ::std::path::PathBuf) { }
	}
}
