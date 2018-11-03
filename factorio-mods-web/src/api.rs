#![allow(
	clippy::single_match_else,
)]

/// Entry-point to the <https://mods.factorio.com/> API
#[derive(Debug)]
pub struct API {
	base_url: reqwest::Url,
	mods_url: reqwest::Url,
	login_url: reqwest::Url,
	client: crate::client::Client,
}

impl API {
	/// Constructs an API object with the given parameters.
	pub fn new(builder: Option<reqwest::r#async::ClientBuilder>) -> crate::Result<Self> {
		Ok(API {
			base_url: BASE_URL.clone(),
			mods_url: MODS_URL.clone(),
			login_url: LOGIN_URL.clone(),
			client: crate::client::Client::new(builder)?,
		})
	}

	/// Searches for mods matching the given criteria.
	pub fn search<'a>(
		&'a self,
		query: &'a str,
	) -> impl futures_core::Stream<Item = crate::Result<crate::SearchResponseMod>> + 'a {
		let query = query.to_lowercase();
		SearchStream {
			query,
			client: &self.client,
			state: SearchStreamState::HavePage(vec![].into_iter(), Some(self.mods_url.clone())),
		}
	}

	/// Gets information about the specified mod.
	pub fn get(&self, mod_name: &factorio_mods_common::ModName) -> impl std::future::Future<Output = crate::Result<crate::Mod>> + 'static {
		let mut mod_url = self.mods_url.clone();
		mod_url.path_segments_mut().unwrap().push(&mod_name.0);
		let future = self.client.get_object(mod_url);

		async {
			let (mod_, _) = await!(future)?;
			Ok(mod_)
		}
	}

	/// Logs in to the web API using the given username and password and returns a credentials object.
	pub fn login(
		&self,
		username: factorio_mods_common::ServiceUsername,
		password: &str,
	) -> impl std::future::Future<Output = crate::Result<factorio_mods_common::UserCredentials>> + 'static {
		// TODO: Replace return type with client::PostObjectFuture

		let future = self.client.post_object(self.login_url.clone(), &[("username", &*username.0), ("password", password)]);

		Box::pinned(async {
			let ((token,), _) = await!(future)?;
			Ok(factorio_mods_common::UserCredentials { username, token })
		})
	}

	/// Downloads the file for the specified mod release and returns a reader to the file contents.
	pub fn download(
		&self,
		release: &crate::ModRelease,
		user_credentials: &factorio_mods_common::UserCredentials,
	) -> impl futures_core::Stream<Item = crate::Result<reqwest::r#async::Chunk>> + 'static {
		let download_url = match self.base_url.join(&release.download_url.0) {
			Ok(mut download_url) => {
				download_url.query_pairs_mut()
					.append_pair("username", &user_credentials.username.0)
					.append_pair("token", &user_credentials.token.0);

				download_url
			},

			Err(err) =>
				return either::Either::Left(futures_util::stream::once(futures_util::future::ready(Err(
					crate::ErrorKind::Parse(format!("{}/{}", self.base_url, release.download_url), err).into())))),
		};

		let future = self.client.get_zip(download_url);

		either::Either::Right(DownloadStream::Fetch(future))
	}
}

// TODO: Use existential type when https://github.com/rust-lang/rust/issues/53443 is fixed
// pub existential type GetResponse: Future<Output = crate::Result<crate::Mod>> + 'static;
// pub existential type DownloadResponse: Stream<Item = crate::Result<reqwest::r#async::Chunk>> + 'static;

#[derive(Debug)]
enum DownloadStream<F> {
	Fetch(F),
	Response(futures_util::compat::Compat01As03<reqwest::r#async::Decoder>, Option<reqwest::Url>),
}

impl<F> futures_core::Stream for DownloadStream<F> where F: std::future::Future<Output = crate::Result<(reqwest::r#async::Response, reqwest::Url)>> {
	type Item = crate::Result<reqwest::r#async::Chunk>;

	fn poll_next(mut self: std::pin::Pin<&mut Self>, lw: &std::task::LocalWaker) -> std::task::Poll<Option<Self::Item>> {
		unsafe {
			loop {
				let (response, download_url) = match std::pin::Pin::get_mut_unchecked(self.as_mut()) {
					DownloadStream::Fetch(f) => match std::pin::Pin::new_unchecked(f).poll(lw) {
						std::task::Poll::Pending => return std::task::Poll::Pending,
						std::task::Poll::Ready(Ok(value)) => value,
						std::task::Poll::Ready(Err(err)) => return std::task::Poll::Ready(Some(Err(err))),
					},

					DownloadStream::Response(body, download_url) => return match std::pin::Pin::new_unchecked(body).poll_next(lw) {
						std::task::Poll::Pending => std::task::Poll::Pending,
						std::task::Poll::Ready(Some(Ok(chunk))) => std::task::Poll::Ready(Some(Ok(chunk))),
						std::task::Poll::Ready(Some(Err(err))) => std::task::Poll::Ready(Some(Err(crate::ErrorKind::HTTP(download_url.take().unwrap(), err).into()))),
						std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
					},
				};

				let body = futures_util::compat::Stream01CompatExt::compat(response.into_body());
				std::pin::Pin::set(self.as_mut(), DownloadStream::Response(body, Some(download_url)));
			}
		}
	}
}

#[derive(Debug)]
struct SearchStream<'a> {
	query: String,
	client: &'a crate::client::Client,
	state: SearchStreamState,
}

enum SearchStreamState {
	WaitingForPage(futures_core::future::LocalFutureObj<'static, crate::Result<(PagedResponse<crate::SearchResponseMod>, reqwest::Url)>>),
	HavePage(std::vec::IntoIter<crate::SearchResponseMod>, Option<reqwest::Url>),
	Ended,
}

impl std::fmt::Debug for SearchStreamState {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match *self {
			SearchStreamState::WaitingForPage(_) =>
				f.debug_tuple("WaitingForPage")
				.finish(),
 			SearchStreamState::HavePage(ref results, ref next_page_url) =>
				f.debug_tuple("HavePage")
				.field(&results.len())
				.field(next_page_url)
				.finish(),
 			SearchStreamState::Ended =>
				f.debug_tuple("Ended")
				.finish(),
		}
	}
}

impl futures_core::Stream for SearchStream<'_> {
	type Item = crate::Result<crate::SearchResponseMod>;

	fn poll_next(mut self: std::pin::Pin<&mut Self>, lw: &std::task::LocalWaker) -> std::task::Poll<Option<Self::Item>> {
		loop {
			let (next_state, result) = match &mut self.state {
				SearchStreamState::WaitingForPage(page) => match std::future::Future::poll(std::pin::Pin::new(page), lw) {
					std::task::Poll::Pending =>
						return std::task::Poll::Pending,

					std::task::Poll::Ready(Ok((page, _))) => (
						Some(SearchStreamState::HavePage(page.results.into_iter(), page.pagination.links.next)),
						None,
					),

					std::task::Poll::Ready(Err(err)) => match err.kind() {
						crate::ErrorKind::StatusCode(_, reqwest::StatusCode::NOT_FOUND) => (
							Some(SearchStreamState::Ended),
							Some(std::task::Poll::Ready(None)),
						),

						_ => (
							Some(SearchStreamState::Ended),
							Some(std::task::Poll::Ready(Some(Err(err)))),
						),
					}
				},

				SearchStreamState::HavePage(results, next_page_url) => match results.next() {
					Some(mod_) => {
						let query = &*self.query;

						if
							mod_.name.0.to_lowercase().contains(query) ||
							mod_.title.0.to_lowercase().contains(query) ||
							mod_.owner.iter().any(|owner| owner.0.to_lowercase().contains(query)) ||
							mod_.summary.0.to_lowercase().contains(query)
						{
							(None, Some(std::task::Poll::Ready(Some(Ok(mod_)))))
						}
						else {
							(None, None)
						}
					},

					None => match next_page_url.take() {
						Some(next_page_url) => (
							Some(SearchStreamState::WaitingForPage(futures_core::future::LocalFutureObj::new(Box::pinned(self.client.get_object(next_page_url))))),
							None,
						),
						None => (
							Some(SearchStreamState::Ended),
							Some(std::task::Poll::Ready(None)),
						),
					},
				},

				SearchStreamState::Ended => (
					Some(SearchStreamState::Ended),
					Some(std::task::Poll::Ready(None)),
				),
			};

			if let Some(next_state) = next_state {
				self.state = next_state;
			}

			if let Some(result) = result {
				return result;
			}
		}
	}
}

/// A single page of a paged response.
#[derive(Debug, serde_derive::Deserialize)]
struct PagedResponse<T> {
	pagination: Pagination,
	results: Vec<T>,
}

/// Pagination information in a paged response.
#[derive(Debug, serde_derive::Deserialize)]
struct Pagination {
	links: PaginationLinks,
}

/// Pagination link information in a paged response.
#[derive(Debug, serde_derive::Deserialize)]
struct PaginationLinks {
	#[serde(deserialize_with = "deserialize_url")]
	next: Option<reqwest::Url>,
}

// TODO: Remove when url supports serde 1.0 (https://github.com/servo/rust-url/pull/327) and reqwest enables or exposes its "serde" feature
fn deserialize_url<'de, D>(deserializer: D) -> Result<Option<reqwest::Url>, D::Error> where D: serde::Deserializer<'de> {
	let url: Option<String> = serde::Deserialize::deserialize(deserializer)?;
	match url {
		Some(url) => match url.parse() {
			Ok(url) => Ok(Some(url)),
			Err(err) => Err(serde::de::Error::custom(format!("invalid URL {:?}: {}", url, std::error::Error::description(&err)))),
		},

		None => Ok(None),
	}
}

lazy_static! {
	static ref BASE_URL: reqwest::Url = "https://mods.factorio.com/".parse().unwrap();
	static ref MODS_URL: reqwest::Url = "https://mods.factorio.com/api/mods?page_size=max".parse().unwrap();
	static ref LOGIN_URL: reqwest::Url = "https://auth.factorio.com/api-login".parse().unwrap();
}

#[cfg(test)]
mod tests {
	use super::*;

	fn run_test<T>(test: T) where for<'r> T: FnOnce(&'r API) -> futures_core::future::LocalFutureObj<'r, ()> {
		use futures_util::FutureExt;

		let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();
		let api = API::new(None).unwrap();
		let result = test(&api).map(|()| Ok::<_, crate::Error>(()));
		runtime.block_on(futures_util::TryFutureExt::compat(Box::pinned(result))).unwrap();
	}

	#[test]
	fn search_list_all_mods() {
		use futures_util::{ FutureExt, StreamExt };

		run_test(|api| futures_core::future::LocalFutureObj::new(Box::pinned(
			api.search("")
			.fold(0usize, |count, result| futures_util::future::ready(count + result.map(|_| 1).unwrap()))
			.map(|count| {
				println!("Found {} mods", count);
				assert!(count > 1700); // 1700+ as of 2017-06-21
			}))));
	}

	#[test]
	fn search_by_title() {
		use futures_util::{ FutureExt, StreamExt };

		run_test(|api| futures_core::future::LocalFutureObj::new(Box::pinned(
			api.search("bob's functions library mod")
			.into_future()
			.map(|(result, _)| {
				let mod_ = result.unwrap().unwrap();
				println!("{:?}", mod_);
				assert_eq!(mod_.title.0, "Bob's Functions Library mod");
			}))));
	}

	#[test]
	fn search_non_existing() {
		use futures_util::{ FutureExt, StreamExt };

		run_test(|api| futures_core::future::LocalFutureObj::new(Box::pinned(
			api.search("arnavion's awesome mod")
			.into_future()
			.map(|(result, _)| assert!(result.is_none())))));
	}

	#[test]
	fn get() {
		use futures_util::FutureExt;

		let mod_name = factorio_mods_common::ModName("boblibrary".to_string());

		run_test(|api| futures_core::future::LocalFutureObj::new(Box::pinned(
			api.get(&mod_name)
			.map(|mod_| {
				let mod_ = mod_.unwrap();
				println!("{:?}", mod_);
				assert_eq!(mod_.title.0, "Bob's Functions Library mod");
			}))));
	}
}
