use ::futures::{ future, Async, Future, Poll, Stream };

pub struct SubCommand;

impl ::util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Show details about specific mods.")
			(@arg mods: ... +required index(1) "mods to show"))
	}

	fn run<'a, 'b, 'c>(
		&'a self,
		matches: &'a ::clap::ArgMatches<'b>,
		_: ::Result<&'c ::factorio_mods_local::API>,
		web_api: ::Result<&'c ::factorio_mods_web::API>,
	) -> Box<Future<Item = (), Error = ::Error> + 'c> where 'a: 'b, 'b: 'c {
		use ::ResultExt;

		let web_api = match web_api {
			Ok(web_api) => web_api,
			Err(err) => return Box::new(future::err(err)),
		};

		let names = matches.values_of("mods").unwrap();
		let names = names.into_iter().map(|name| ::factorio_mods_common::ModName::new(name.to_string()));

		Box::new(
			futures_ordered(
				names.map(move |name|
					web_api.get(&name)
					.or_else(move |err| Err(err).chain_err(|| format!("Could not retrieve mod {}", name)))))
			.for_each(|mod_| {
				println!("Name: {}", mod_.name());
				println!("Author: {}", ::itertools::join(mod_.owner(), ", "));
				println!("Title: {}", mod_.title());
				println!("Summary: {}", mod_.summary());
				println!("Description:");
				for line in mod_.description().lines() {
					println!("    {}", line);
				}

				println!("Tags: {}", ::itertools::join(mod_.tags().iter().map(|t| t.name()), ", "));

				let homepage = mod_.homepage();
				if !homepage.is_empty() {
					println!("Homepage: {}", homepage);
				}

				let github_path = mod_.github_path();
				if !github_path.is_empty() {
					println!("GitHub page: https://github.com/{}", github_path);
				}

				println!("License: {}", mod_.license_name());

				println!("Game versions: {}", ::itertools::join(mod_.game_versions(), ", "));

				println!("Releases:");
				let releases = mod_.releases();
				if releases.is_empty() {
					println!("    No releases");
				}
				else {
					for release in releases {
						println!("    Version: {:-9} Game version: {:-9}", release.version(), release.factorio_version());
					}
				}

				println!("");

				Ok(())
			}))
	}
}

// TODO: Should be replaced with `::futures::stream::futures_ordered()` when that's released
fn futures_ordered<I>(futures: I) -> impl Stream<Item = <I::Item as Future>::Item, Error = <I::Item as Future>::Error> where I: IntoIterator, I::Item: Future {
	let futures = futures.into_iter().map(FuturesOrderedElement::Pending).collect();
	FuturesOrdered(futures)
}

struct FuturesOrdered<F>(::std::collections::VecDeque<FuturesOrderedElement<F>>) where F: Future;

enum FuturesOrderedElement<F> where F: Future {
	Pending(F),
	Completed(Result<F::Item, F::Error>),
	Invalid,
}

impl<F> Stream for FuturesOrdered<F> where F: Future {
	type Item = F::Item;
	type Error = F::Error;

	fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
		for pending_future in &mut self.0 {
			*pending_future = match ::std::mem::replace(pending_future, FuturesOrderedElement::Invalid) {
				FuturesOrderedElement::Pending(mut f) => match f.poll() {
					Ok(Async::Ready(value)) => FuturesOrderedElement::Completed(Ok(value)),
					Ok(Async::NotReady) => FuturesOrderedElement::Pending(f),
					Err(err) => FuturesOrderedElement::Completed(Err(err)),
				},

				FuturesOrderedElement::Completed(r) =>
					FuturesOrderedElement::Completed(r),

				FuturesOrderedElement::Invalid =>
					unreachable!(),
			};
		}

		match self.0.pop_front() {
			Some(FuturesOrderedElement::Pending(f)) => {
				self.0.push_front(FuturesOrderedElement::Pending(f));
				Ok(Async::NotReady)
			},

			Some(FuturesOrderedElement::Completed(r)) =>
				r.map(|v| Async::Ready(Some(v))),

			Some(FuturesOrderedElement::Invalid) =>
				unreachable!(),

			None =>
				Ok(Async::Ready(None)),
		}
	}
}
