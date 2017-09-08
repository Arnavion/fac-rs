use ::futures::{ Async, Future, Poll, Stream };

/// The page number of one page of a search response.
#[derive(
	Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	::derive_new::new,
	::derive_struct::newtype_display, ::derive_struct::newtype_ref,
	::serde_derive::Deserialize,
)]
pub struct PageNumber(u64);

/// The response number within a page of a search response.
#[derive(
	Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	::derive_new::new,
	::derive_struct::newtype_display, ::derive_struct::newtype_ref,
	::serde_derive::Deserialize,
)]
pub struct ResponseNumber(u64);

/// A mod object returned by `API::search`.
#[derive(Clone, Debug, PartialEq, ::derive_new::new, ::derive_struct::getters, ::serde_derive::Deserialize)]
pub struct SearchResponseMod {
	/// The mod ID.
	#[getter(copy)]
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
	#[getter(copy)]
	downloads_count: ::DownloadCount,

	/// The tags of the mod.
	#[serde(deserialize_with = "::factorio_mods_common::deserialize_string_or_seq_string")]
	tags: Vec<::Tag>,
}

/// Constructs an stream of search results.
pub fn search<'a>(
	client: &'a ::client::Client,
	url: ::reqwest::Url,
) -> impl Stream<Item = ::SearchResponseMod, Error = ::Error> + 'a {
	let page_future = client.get_object(url);

	SearchResultsStream {
		client,
		state: SearchResultsStreamState::WaitingForPage(Box::new(page_future)),
	}
}

/// A stream of search results.
#[derive(Debug)]
struct SearchResultsStream<'a> {
	client: &'a ::client::Client,
	state: SearchResultsStreamState<'a>,
}

enum SearchResultsStreamState<'a> {
	WaitingForPage(Box<Future<Item = (SearchResponse, ::reqwest::Url), Error = ::Error> + 'a>),
	HavePage(::std::vec::IntoIter<SearchResponseMod>, Option<::reqwest::Url>),
	Ended,
}

impl<'a> ::std::fmt::Debug for SearchResultsStreamState<'a> {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match *self {
			SearchResultsStreamState::WaitingForPage(_) =>
				f.debug_tuple("WaitingForPage")
				.finish(),

			SearchResultsStreamState::HavePage(ref results, ref next_page_url) =>
				f.debug_tuple("HavePage")
				.field(&results.len())
				.field(next_page_url)
				.finish(),

			SearchResultsStreamState::Ended =>
				f.debug_tuple("Ended")
				.finish(),
		}
	}
}

impl<'a> Stream for SearchResultsStream<'a> {
	type Item = SearchResponseMod;
	type Error = ::Error;

	fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
		let mut state = ::std::mem::replace(&mut self.state, SearchResultsStreamState::Ended);

		let (next_state, result) = loop {
			state = match state {
				SearchResultsStreamState::WaitingForPage(mut page_future) => match page_future.poll() {
					Ok(Async::Ready((page, _))) =>
						SearchResultsStreamState::HavePage(page.results.into_iter(), page.pagination.links.next),

					Ok(Async::NotReady) => break (
						SearchResultsStreamState::WaitingForPage(page_future),
						Ok(Async::NotReady)),

					Err(::Error(::ErrorKind::StatusCode(_, ::reqwest::StatusCode::NotFound), _)) => break (
						SearchResultsStreamState::Ended,
						Ok(Async::Ready(None))),

					Err(err) => break (
						SearchResultsStreamState::Ended,
						Err(err)),
				},

				SearchResultsStreamState::HavePage(mut results, next_page_url) => match (results.next(), next_page_url) {
					(Some(mod_), next_page_url) => break (
						SearchResultsStreamState::HavePage(results, next_page_url),
						Ok(Async::Ready(Some(mod_)))),

					(None, Some(next_page_url)) =>
						SearchResultsStreamState::WaitingForPage(Box::new(self.client.get_object(next_page_url))),

					(None, None) => break (
						SearchResultsStreamState::Ended,
						Ok(Async::Ready(None))),
				},

				SearchResultsStreamState::Ended => break (
					SearchResultsStreamState::Ended,
					Ok(Async::Ready(None))),
			};
		};

		self.state = next_state;

		result
	}
}

/// A single search response.
#[derive(Debug, ::serde_derive::Deserialize)]
struct SearchResponse {
	pagination: SearchResponsePagination,
	results: Vec<SearchResponseMod>,
}

/// Pagination information in a search response.
#[derive(Debug, ::serde_derive::Deserialize)]
struct SearchResponsePagination {
	page_count: PageNumber,
	page: PageNumber,

	page_size: ResponseNumber,
	count: ResponseNumber,

	links: SearchResponsePaginationLinks,
}

/// Pagination link information in a search response.
#[derive(Debug, ::serde_derive::Deserialize)]
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

// TODO: Remove when url supports serde 1.0 (https://github.com/servo/rust-url/pull/327) and reqwest enables or exposes its "serde" feature
fn deserialize_url<'de, D>(deserializer: D) -> Result<Option<::reqwest::Url>, D::Error> where D: ::serde::Deserializer<'de> {
	let url: Option<String> = ::serde::Deserialize::deserialize(deserializer)?;
	match url {
		Some(url) => match url.parse() {
			Ok(url) => Ok(Some(url)),
			Err(err) => Err(::serde::de::Error::custom(format!("invalid URL {:?}: {}", url, ::std::error::Error::description(&err)))),
		},

		None => Ok(None),
	}
}
