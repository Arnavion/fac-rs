make_newtype!(pub PageNumber(u64));

make_deserializable!(struct ResponseNumber(u64));

make_deserializable!(struct SearchResponse {
	pagination: SearchResponsePagination,
	results: Vec<SearchResponseMod>,
});

make_deserializable!(struct SearchResponsePagination {
	page_count: PageNumber,
	page: PageNumber,

	page_size: ResponseNumber,
	count: ResponseNumber,

	links: SearchResponsePaginationLinks,
});

make_deserializable!(struct SearchResponsePaginationLinks {
	prev: Option<String>,
	next: Option<String>,
	first: Option<String>,
	last: Option<String>,
});

const BASE_URL: &'static str = "https://mods.factorio.com/api/";
const LOGIN_URL: &'static str = "https://auth.factorio.com/api-login";
const DEFAULT_PAGE_SIZE: PageNumber = PageNumber(25);
const DEFAULT_ORDER: &'static str = "top";

#[derive(Debug)]
pub struct API {
	base_url: String,
	login_url: String,
	url: ::hyper::Url,
	client: ::hyper::Client,
}

make_deserializable!(pub struct SearchResponseMod {
	pub id: ::factorio_mods_common::ModId,

	pub name: ::factorio_mods_common::ModName,
	pub owner: ::factorio_mods_common::AuthorNames,
	pub title: ::factorio_mods_common::ModTitle,
	pub summary: ::factorio_mods_common::ModSummary,

	pub github_path: ::factorio_mods_common::Url,
	pub homepage: ::factorio_mods_common::Url,
	pub license_name: ::factorio_mods_common::LicenseName,
	pub license_url: ::factorio_mods_common::Url,

	pub game_versions: Vec<::factorio_mods_common::GameVersion>,

	pub created_at: ::factorio_mods_common::DateTime,
	pub updated_at: ::factorio_mods_common::DateTime,
	pub latest_release: ::factorio_mods_common::ModRelease,

	pub current_user_rating: Option<::serde_json::Value>,
	pub downloads_count: ::factorio_mods_common::DownloadCount,
	pub visits_count: ::factorio_mods_common::VisitCount,
	pub tags: ::factorio_mods_common::Tags,
});

#[derive(Debug)]
pub struct SearchResultsIterator<'a> {
	client: &'a ::hyper::Client,
	url: ::hyper::Url,
	current_page_number: PageNumber,
	current_page: Option<SearchResponse>,
	errored: bool,
}

impl<'a> Iterator for SearchResultsIterator<'a> {
	type Item = ::error::Result<SearchResponseMod>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.errored {
			return None;
		}

		match self.current_page {
			Some(ref mut page) if !page.results.is_empty() => {
				let result = page.results.remove(0);
				return Some(Ok(result));
			}

			Some(_) => {
				self.current_page_number.0 += 1;
				self.current_page = None;
				return self.next()
			},

			None => {
				let mut url = self.url.clone();

				url.query_pairs_mut().append_pair("page", &self.current_page_number.to_string());

				match get_object(self.client, url) {
					Ok(page) => {
						self.current_page = Some(page);
						return self.next();
					},

					Err(err) => match err.kind() {
						&::error::ErrorKind::StatusCode(::hyper::status::StatusCode::NotFound) => {
							self.errored = true;
							return None;
						}

						_ => {
							self.errored = true;
							return Some(Err(err));
						}
					},
				}
			}
		}
	}
}

impl API {
	pub fn new(base_url: Option<String>, login_url: Option<String>, client: Option<::hyper::Client>) -> ::error::Result<API> {
		let base_url = base_url.unwrap_or_else(|| BASE_URL.to_string());
		let login_url = login_url.unwrap_or_else(|| LOGIN_URL.to_string());

		let url = base_url.trim_right_matches('/').to_string() + "/mods";
		let url = try!(::hyper::Url::parse(&url));
		if url.cannot_be_a_base() {
			return Err(format!("url {} cannot be a base.", url).into());
		}

		let client = client.unwrap_or_else(::hyper::Client::new);

		Ok(API {
			base_url: base_url,
			login_url: login_url,
			url: url,
			client: client,
		})
	}

	pub fn search<'a>(&'a self, query: &str, tags: &Vec<&::factorio_mods_common::TagName>, order: Option<String>, page_size: Option<PageNumber>, page: Option<PageNumber>) -> ::error::Result<SearchResultsIterator<'a>> {
		let tags_query = ::itertools::join(tags, ",");
		let order = order.unwrap_or_else(|| DEFAULT_ORDER.to_string());
		let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE).0.to_string();
		let page = page.unwrap_or_else(|| PageNumber(1));

		let mut url = self.url.clone();
		url.query_pairs_mut()
			.append_pair("q", query)
			.append_pair("tags", &tags_query)
			.append_pair("order", &order)
			.append_pair("page_size", &page_size);

		Ok(SearchResultsIterator {
			client: &self.client,
			url: url,
			current_page_number: page,
			current_page: None,
			errored: false,
		})
	}

	pub fn get(&self, mod_name: ::factorio_mods_common::ModName) -> ::error::Result<::factorio_mods_common::Mod> {
		let mut url = self.url.clone();
		url.path_segments_mut().unwrap().push(&mod_name);
		get_object(&self.client, url)
	}
}

fn get(client: &::hyper::Client, url: ::hyper::Url) -> ::error::Result<::hyper::client::Response> {
	let response = try!(client.get(url).send());
	match response.status {
		::hyper::status::StatusCode::Ok => Ok(response),
		code => Err(::error::ErrorKind::StatusCode(code).into())
	}
}

fn get_object<T>(client: &::hyper::Client, url: ::hyper::Url) -> ::error::Result<T> where T: ::serde::Deserialize {
	let response = try!(get(client, url));
	let object = try!(::serde_json::from_reader(response));
	return Ok(object);
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn search_list_all_mods() {
		let api = API::new(None, None, None).unwrap();

		let iter = api.search("", &vec![], None, None, None).unwrap();
		let mods = iter.map(|m| m.unwrap()); // Ensure all are Ok()
		let count = mods.count();
		println!("Found {} mods", count);
		assert!(count > 500); // 700+ as of 2016-10-03
	}

	#[test]
	fn search_by_title() {
		let api = API::new(None, None, None).unwrap();

		let mut iter = api.search("bob's functions library mod", &vec![], None, None, None).unwrap();
		let mod_ = iter.next().unwrap().unwrap();
		println!("{:?}", mod_);
		assert!(mod_.title.0 == "Bob's Functions Library mod");
	}

	#[test]
	fn search_by_tag() {
		let api = API::new(None, None, None).unwrap();

		let mut iter = api.search("", &vec![&::factorio_mods_common::TagName("logistics".to_string())], None, None, None).unwrap();
		let mod_ = iter.next().unwrap().unwrap();
		println!("{:?}", mod_);
		let mut tags = mod_.tags.0.iter().filter(|tag| tag.name.0 == "logistics");
		let tag = tags.next().unwrap();
		println!("{:?}", tag);
	}

	#[test]
	fn get() {
		let api = API::new(None, None, None).unwrap();

		let mod_ = api.get(::factorio_mods_common::ModName("boblibrary".to_string())).unwrap();
		println!("{:?}", mod_);
		assert!(mod_.title.0 == "Bob's Functions Library mod");
	}
}
