/// The page number of one page of a search response.
#[derive(newtype)]
pub struct PageNumber(u64);

/// The response number within a page of a search response.
#[derive(newtype)]
pub struct ResponseNumber(u64);

/// A mod object returned by `API::search`.
#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct SearchResponseMod {
	id: ::ModId,

	name: ::factorio_mods_common::ModName,
	owner: ::factorio_mods_common::AuthorNames,
	title: ::factorio_mods_common::ModTitle,
	summary: ::ModSummary,

	github_path: ::factorio_mods_common::Url,
	homepage: ::factorio_mods_common::Url,
	license_name: ::LicenseName,
	license_url: ::factorio_mods_common::Url,

	game_versions: Vec<::factorio_mods_common::GameVersion>,

	created_at: ::DateTime,
	updated_at: ::DateTime,
	latest_release: ::ModRelease,

	current_user_rating: Option<::serde_json::Value>,
	downloads_count: ::DownloadCount,
	visits_count: ::VisitCount,
	tags: ::Tags,
}

/// An iterator of search results.
#[derive(Debug)]
pub struct SearchResultsIterator<'a> {
	client: &'a ::hyper::Client,
	mods_url: ::hyper::Url,
	current_page_number: PageNumber,
	current_page: Option<SearchResponse>,
	errored: bool,
}

impl<'a> SearchResultsIterator<'a> {
	/// Constructs an iterator of search results.
	pub fn new(
		client: &'a ::hyper::Client,
		mods_url: ::hyper::Url,
		starting_page_number: ::PageNumber,
	) -> SearchResultsIterator<'a> {
		SearchResultsIterator {
			client: client,
			mods_url: mods_url,
			current_page_number: starting_page_number,
			current_page: None,
			errored: false,
		}
	}
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

					Err(err) => match *err.kind() {
						::ErrorKind::StatusCode(::hyper::status::StatusCode::NotFound) => {
							self.errored = true;
							None
						},

						_ => {
							self.errored = true;
							Some(Err(err))
						},
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
