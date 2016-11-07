#[derive(newtype)]
pub struct PageNumber(u64);

#[derive(newtype)]
pub struct ResponseNumber(u64);

#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct SearchResponseMod {
	id: ::factorio_mods_common::ModId,

	name: ::factorio_mods_common::ModName,
	owner: ::factorio_mods_common::AuthorNames,
	title: ::factorio_mods_common::ModTitle,
	summary: ::factorio_mods_common::ModSummary,

	github_path: ::factorio_mods_common::Url,
	homepage: ::factorio_mods_common::Url,
	license_name: ::factorio_mods_common::LicenseName,
	license_url: ::factorio_mods_common::Url,

	game_versions: Vec<::factorio_mods_common::GameVersion>,

	created_at: ::factorio_mods_common::DateTime,
	updated_at: ::factorio_mods_common::DateTime,
	latest_release: ::factorio_mods_common::ModRelease,

	current_user_rating: Option<::serde_json::Value>,
	downloads_count: ::factorio_mods_common::DownloadCount,
	visits_count: ::factorio_mods_common::VisitCount,
	tags: ::factorio_mods_common::Tags,
}

#[derive(Debug)]
pub struct SearchResultsIterator<'a> {
	client: &'a ::hyper::Client,
	mods_url: ::hyper::Url,
	current_page_number: PageNumber,
	current_page: Option<SearchResponse>,
	errored: bool,
}

impl<'a> SearchResultsIterator<'a> {
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
			}

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
						}

						_ => {
							self.errored = true;
							Some(Err(err))
						}
					},
				}
			}
		}
	}
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
	pagination: SearchResponsePagination,
	results: Vec<SearchResponseMod>,
}

#[derive(Debug, Deserialize)]
struct SearchResponsePagination {
	page_count: PageNumber,
	page: PageNumber,

	page_size: ResponseNumber,
	count: ResponseNumber,

	links: SearchResponsePaginationLinks,
}

#[derive(Debug, Deserialize)]
struct SearchResponsePaginationLinks {
	prev: Option<String>,
	next: Option<String>,
	first: Option<String>,
	last: Option<String>,
}
