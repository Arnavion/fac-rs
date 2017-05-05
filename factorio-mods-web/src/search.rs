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
	game_versions: Vec<::factorio_mods_common::ModVersionReq>,

	/// The date and time at which the mod was created.
	created_at: ::DateTime,

	/// The date and time at which the mod was last updated.
	updated_at: ::DateTime,

	/// The latest release of the mod.
	latest_release: ::ModRelease,

	// current_user_rating: ???, # Unknown type

	/// The number of times the mod has been downloaded.
	downloads_count: ::DownloadCount,

	/// The tags of the mod.
	#[serde(deserialize_with = "::factorio_mods_common::deserialize_string_or_seq_string")]
	tags: Vec<::Tag>,
}

/// Constructs an iterator of search results.
pub fn search<'a>(client: &'a ::client::Client, url: ::reqwest::Url) -> impl Iterator<Item = ::Result<::SearchResponseMod>> + 'a {
	SearchResultsIterator {
		client,
		url,
		current_page: None,
		ended: false,
	}
}

/// An iterator of search results.
#[derive(Debug)]
struct SearchResultsIterator<'a> {
	client: &'a ::client::Client,
	url: ::reqwest::Url,
	current_page: Option<SearchResponse>,
	ended: bool,
}

impl<'a> Iterator for SearchResultsIterator<'a> {
	type Item = ::Result<SearchResponseMod>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.ended {
			return None;
		}

		if let Some(mut page) = self.current_page.take() {
			if page.results.is_empty() {
				if let Some(next_url) = page.pagination.links.next {
					self.url = next_url;
					self.next()
				}
				else {
					self.ended = true;
					None
				}
			}
			else {
				let result = page.results.remove(0);
				self.current_page = Some(page);
				Some(Ok(result))
			}
		}
		else {
			match self.client.get_object(self.url.clone()) {
				Ok(page) => {
					self.current_page = Some(page);
					self.next()
				},

				Err(::Error(::ErrorKind::StatusCode(_, ::reqwest::StatusCode::NotFound), _)) => {
					self.ended = true;
					None
				},

				Err(err) => {
					self.ended = true;
					Some(Err(err))
				},
			}
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
	#[serde(deserialize_with = "deserialize_url")]
	prev: Option<::reqwest::Url>,

	#[serde(deserialize_with = "deserialize_url")]
	next: Option<::reqwest::Url>,

	#[serde(deserialize_with = "deserialize_url")]
	first: Option<::reqwest::Url>,

	#[serde(deserialize_with = "deserialize_url")]
	last: Option<::reqwest::Url>,
}

/// Deserializes a URL.
pub fn deserialize_url<D>(deserializer: D) -> Result<Option<::reqwest::Url>, D::Error> where D: ::serde::Deserializer {
	let url: Option<String> = ::serde::Deserialize::deserialize(deserializer)?;
	if let Some(url) = url {
		::reqwest::Url::parse(&url).map(Some)
		.map_err(|err| ::serde::de::Error::custom(format!("invalid URL {:?}: {}", &url, ::std::error::Error::description(&err))))
	}
	else {
		Ok(None)
	}
}
