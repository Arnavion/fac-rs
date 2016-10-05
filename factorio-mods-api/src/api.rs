use types;

make_deserializable!(pub struct PageNumber(pub u64));

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
	pub id: types::ModId,

	pub name: types::ModName,
	pub owner: types::AuthorNames,
	pub title: types::ModTitle,
	pub summary: types::ModSummary,

	pub github_path: types::Url,
	pub homepage: types::Url,

	pub game_versions: Vec<types::GameVersion>,

	pub created_at: types::DateTime,
	pub latest_release: types::ModRelease,

	pub current_user_rating: Option<::serde_json::Value>,
	pub downloads_count: types::DownloadCount,
	pub tags: Vec<types::Tag>,
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
	type Item = Result<SearchResponseMod, APIError>;

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

				url.query_pairs_mut().append_pair("page", self.current_page_number.0.to_string().as_str());

				match get_object(self.client, url) {
					Ok(page) => {
						self.current_page = Some(page);
						return self.next();
					},

					Err(APIError::StatusCode { status_code: ::hyper::status::StatusCode::NotFound, .. }) => {
						self.errored = true;
						return None;
					},

					Err(err) => {
						self.errored = true;
						return Some(Err(err));
					},
				}
			}
		}
	}
}

impl API {
	pub fn new(base_url: Option<String>, login_url: Option<String>, client: Option<::hyper::Client>) -> Result<API, APIError> {
		let base_url = base_url.unwrap_or_else(|| BASE_URL.to_string());
		let login_url = login_url.unwrap_or_else(|| LOGIN_URL.to_string());
		let url = base_url.trim_right_matches('/').to_string() + "/mods";
		let url = try!(::hyper::Url::parse(url.as_str()).map_err(APIError::parse));
		let client = client.unwrap_or_else(::hyper::Client::new);

		Ok(API {
			base_url: base_url,
			login_url: login_url,
			url: url,
			client: client,
		})
	}

	pub fn search<'a>(&'a self, query: &str, tags: &Vec<&types::TagName>, order: Option<String>, page_size: Option<PageNumber>, page: Option<PageNumber>) -> Result<SearchResultsIterator<'a>, APIError> {
		let tags_query = ::itertools::join(tags.iter().map(|t| &t.0), ",");
		let order = order.unwrap_or_else(|| DEFAULT_ORDER.to_string());
		let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE).0.to_string();
		let page = page.unwrap_or_else(|| PageNumber(1));

		let mut url = self.url.clone();
		url.query_pairs_mut()
			.append_pair("q", query)
			.append_pair("tags", tags_query.as_str())
			.append_pair("order", order.as_str())
			.append_pair("page_size", page_size.as_str());

		Ok(SearchResultsIterator {
			client: &self.client,
			url: url,
			current_page_number: page,
			current_page: None,
			errored: false,
		})
	}

	pub fn get(&self, mod_name: types::ModName) -> Result<types::Mod, APIError> {
		let mut url = self.url.clone();
		try!(url.path_segments_mut().map_err(|_| APIError::other())).push(mod_name.0.as_str());
		get_object(&self.client, url)
	}
}

#[derive(Debug)]
pub enum APIError {
	Hyper { error: ::hyper::Error, backtrace: Option<::backtrace::Backtrace> },
	Parse { error: ::url::ParseError, backtrace: Option<::backtrace::Backtrace> },
	StatusCode { status_code: ::hyper::status::StatusCode, backtrace: Option<::backtrace::Backtrace> },
	JSON { error: ::serde_json::Error, backtrace: Option<::backtrace::Backtrace> },
	Other { backtrace: Option<::backtrace::Backtrace> },
}

impl APIError {
	fn hyper(error: ::hyper::Error) -> APIError {
		APIError::Hyper { error: error, backtrace: APIError::backtrace() }
	}

	fn parse(error: ::url::ParseError) -> APIError {
		APIError::Parse { error: error, backtrace: APIError::backtrace() }
	}

	fn status_code(status_code: ::hyper::status::StatusCode) -> APIError {
		APIError::StatusCode { status_code: status_code, backtrace: APIError::backtrace() }
	}

	fn json(error: ::serde_json::Error) -> APIError {
		APIError::JSON { error: error, backtrace: APIError::backtrace() }
	}

	fn other() -> APIError {
		APIError::Other { backtrace: APIError::backtrace() }
	}

	fn backtrace() -> Option<::backtrace::Backtrace> {
		::std::env::var("RUST_BACKTRACE").ok()
			.and_then(|value| { if value == "1" { Some(::backtrace::Backtrace::new()) } else { None } })
	}
}

fn get(client: &::hyper::Client, url: ::hyper::Url) -> Result<::hyper::client::Response, APIError> {
	let response = try!(client.get(url).send().map_err(APIError::hyper));
	match response.status {
		::hyper::status::StatusCode::Ok => Ok(response),
		code => Err(APIError::status_code(code))
	}
}

fn get_object<T>(client: &::hyper::Client, url: ::hyper::Url) -> Result<T, APIError> where T: ::serde::Deserialize {
	let response = try!(get(client, url));
	let object = try!(::serde_json::from_reader(response).map_err(APIError::json));
	return Ok(object);
}


#[test]
fn test_search_list_all_mods() {
	let api = API::new(None, None, None).unwrap();

	let iter = api.search("", &vec![], None, None, None).unwrap();
	let count = iter.count();
	println!("Found {} mods", count);
	assert!(count > 500); // 700+ as of 2016-10-03
}

#[test]
fn test_search_by_title() {
	let api = API::new(None, None, None).unwrap();

	let mut iter = api.search("bob's functions library mod", &vec![], None, None, None).unwrap();
	let mod_ = iter.next().unwrap().unwrap();
	println!("{:?}", mod_);
	assert!(mod_.title.0 == "Bob's Functions Library mod");
}

#[test]
fn test_search_by_tag() {
	let api = API::new(None, None, None).unwrap();

	let mut iter = api.search("", &vec![&types::TagName("logistics".to_string())], None, None, None).unwrap();
	let mod_ = iter.next().unwrap().unwrap();
	println!("{:?}", mod_);
	let mut tags = mod_.tags.iter().filter(|tag| tag.name.0 == "logistics");
	let tag = tags.next().unwrap();
	println!("{:?}", tag);
}

#[test]
fn test_get() {
	let api = API::new(None, None, None).unwrap();

	let mod_ = api.get(types::ModName("boblibrary".to_string())).unwrap();
	println!("{:?}", mod_);
	assert!(mod_.title.0 == "Bob's Functions Library mod");
}
