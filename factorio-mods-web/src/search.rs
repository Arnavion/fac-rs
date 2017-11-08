use ::futures::Stream;

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
	::async_stream_block! {
		let mut next_page_url = Some(url);

		while let Some(url) = next_page_url {
			match ::await!(client.get_object::<SearchResponse>(url)) {
				Ok((page, _)) => {
					for mod_ in page.results {
						::stream_yield!(mod_);
					}

					next_page_url = page.pagination.links.next;
				},

				Err(::Error(::ErrorKind::StatusCode(_, ::reqwest::StatusCode::NotFound), _)) =>
					break,

				Err(err) => return Err(err),
			}
		}

		Ok(())
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
