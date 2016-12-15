/// The page number of one page of a search response.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct PageNumber(u64);

/// The response number within a page of a search response.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct ResponseNumber(u64);

/// A mod object returned by `API::search`.
#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct SearchResponseMod {
	/// The mod ID.
	id: ::ModId,

	/// The name of the mod.
	name: ::factorio_mods_common::ModName,

	/// The authors of the mod.
	#[serde(deserialize_with = "::factorio_mods_common::deserialize_string_or_seq_string")]
	owner: Vec<::factorio_mods_common::AuthorName>,

	/// The title of the mod.
	title: ::factorio_mods_common::ModTitle,

	/// A short summary of the mod.
	summary: ::ModSummary,

	/// The URL of the GitHub repository of the mod.
	github_path: ::factorio_mods_common::Url,

	/// The URL of the homepage of the mod.
	homepage: ::factorio_mods_common::Url,

	/// The name of the mod's license.
	license_name: ::LicenseName,

	/// The URL of the mod's license.
	license_url: ::factorio_mods_common::Url,

	/// The versions of the game supported by the mod.
	game_versions: Vec<::factorio_mods_common::GameVersion>,

	/// The date and time at which the mod was created.
	created_at: ::DateTime,

	/// The date and time at which the mod was last updated.
	updated_at: ::DateTime,

	/// The latest release of the mod.
	latest_release: ::ModRelease,

	// current_user_rating: ???, # Unknown type

	/// The number of times the mod has been downloaded.
	downloads_count: ::DownloadCount,

	/// The number of times the mod page has been visited.
	visits_count: ::VisitCount,

	/// The tags of the mod.
	#[serde(deserialize_with = "::factorio_mods_common::deserialize_string_or_seq_string")]
	tags: Vec<::Tag>,
}

/// Constructs an iterator of search results.
pub fn search<'a>(
	client: &'a ::reqwest::Client,
	mods_url: ::reqwest::Url,
	starting_page_number: ::PageNumber,
) -> impl Iterator<Item = ::Result<::SearchResponseMod>> + 'a {
	SearchResultsIterator {
		client: client,
		mods_url: mods_url,
		current_page_number: starting_page_number,
		current_page: None,
		errored: false,
	}
}

/// An iterator of search results.
#[derive(Debug)]
struct SearchResultsIterator<'a> {
	client: &'a ::reqwest::Client,
	mods_url: ::reqwest::Url,
	current_page_number: PageNumber,
	current_page: Option<SearchResponse>,
	errored: bool,
}

impl<'a> Iterator for SearchResultsIterator<'a> {
	type Item = ::Result<SearchResponseMod>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.errored {
			return None;
		}

		match self.current_page {
			Some(ref mut page) if !page.results.is_empty() => {
				let result = page.results.remove(0);
				Some(Ok(result))
			},

			Some(_) => {
				*self.current_page_number += 1;
				self.current_page = None;
				self.next()
			},

			None => {
				let mut mods_url = self.mods_url.clone();
				mods_url.query_pairs_mut().append_pair("page", &self.current_page_number.to_string());

				match ::util::get_object(self.client, mods_url) {
					Ok(page) => {
						self.current_page = Some(page);
						self.next()
					},

					Err(::Error(::ErrorKind::StatusCode(::reqwest::StatusCode::NotFound), _)) => {
						self.errored = true;
						None
					},

					Err(err) => {
						self.errored = true;
						Some(Err(err))
					},
				}
			},
		}
	}
}

/// A single search response.
#[derive(Debug, Deserialize)]
struct SearchResponse {
	pagination: SearchResponsePagination,
	results: Vec<SearchResponseMod>,
}

/// Pagination information in a search response.
#[derive(Debug, Deserialize)]
struct SearchResponsePagination {
	page_count: PageNumber,
	page: PageNumber,

	page_size: ResponseNumber,
	count: ResponseNumber,

	links: SearchResponsePaginationLinks,
}

/// Pagination link information in a search response.
#[derive(Debug, Deserialize)]
struct SearchResponsePaginationLinks {
	prev: Option<String>,
	next: Option<String>,
	first: Option<String>,
	last: Option<String>,
}
