extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate url;

use types;

make_deserializable!(pub struct PageNumber(u64));

make_deserializable!(struct ResponseNumber(u64));

make_deserializable!(struct SearchResponse {
	pagination: SearchResponsePagination,
	results: Vec<types::Mod>,
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
	url: String,
	client: hyper::Client,
}

#[derive(Debug)]
pub struct SearchResultsIterator<'a> {
	client: &'a hyper::Client,
	url: hyper::Url,
	current_page_number: PageNumber,
	current_page: Option<SearchResponse>,
	errored: bool,
}

impl<'a> Iterator for SearchResultsIterator<'a> {
	type Item = Result<types::Mod, APIError>;

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

					Err(APIError::StatusCode(hyper::status::StatusCode::NotFound)) => {
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
	pub fn new(base_url: Option<String>, login_url: Option<String>, client: Option<hyper::Client>) -> API {
		let base_url = base_url.unwrap_or_else(|| BASE_URL.to_string());
		let login_url = login_url.unwrap_or_else(|| LOGIN_URL.to_string());
		let url = base_url.trim_right_matches('/').to_string() + "/mods";
		let client = client.unwrap_or_else(hyper::Client::new);

		API {
			base_url: base_url,
			login_url: login_url,
			url: url,
			client: client,
		}
	}

	pub fn search<'a>(&'a self, query: &str, tags: Vec<&str>, order: Option<String>, page_size: Option<PageNumber>, page: Option<PageNumber>) -> Result<SearchResultsIterator<'a>, APIError> {
		let tags_query = tags.join(",");
		let order = order.unwrap_or_else(|| DEFAULT_ORDER.to_string());
		let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE).0.to_string();
		let page = page.unwrap_or_else(|| PageNumber(1));

		let mut url = try!(hyper::Url::parse(self.url.as_str()).map_err(APIError::Parse));
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
}

#[derive(Debug)]
pub enum APIError {
	Hyper(hyper::Error),
	Parse(url::ParseError),
	StatusCode(hyper::status::StatusCode),
	JSON(serde_json::Error),
}

fn get(client: &hyper::Client, url: hyper::Url) -> Result<hyper::client::Response, APIError> {
	let response = try!(client.get(url).send().map_err(APIError::Hyper));
	match response.status {
		hyper::status::StatusCode::Ok => Ok(response),
		code => Err(APIError::StatusCode(code))
	}
}

fn get_object<T>(client: &hyper::Client, url: hyper::Url) -> Result<T, APIError> where T: serde::Deserialize {
	let response = try!(get(client, url));
	let object = try!(serde_json::from_reader(response).map_err(APIError::JSON));
	return Ok(object);
}
