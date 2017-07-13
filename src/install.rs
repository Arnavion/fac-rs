use ::futures::{ future, Future, IntoFuture };

lazy_static! {
	static ref REQUIREMENT_REGEX: ::regex::Regex = ::regex::Regex::new(r"^([^@]+)(?:@(.*))?").unwrap();
}

pub struct SubCommand;

impl ::util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Install (or update) mods.")
			(@arg requirements: ... +required index(1) "requirements to install"))
	}

	fn run<'a, 'b, 'c>(
		&'a self,
		matches: &'a ::clap::ArgMatches<'b>,
		local_api: ::Result<&'c ::factorio_mods_local::API>,
		web_api: ::Result<&'c ::factorio_mods_web::API>,
	) -> Box<Future<Item = (), Error = ::Error> + 'c> where 'a: 'b, 'b: 'c {
		use ::ResultExt;

		let requirements = matches.values_of("requirements").unwrap();

		let (local_api, web_api) = match (local_api, web_api) {
			(Ok(local_api), Ok(web_api)) => (local_api, web_api),
			(Err(err), _) | (_, Err(err)) => return Box::new(future::err(err)),
		};

		let config = match ::config::Config::load(local_api) {
			Ok(config) => config,
			Err(err) => return Box::new(future::err(err)),
		};

		let mut reqs = config.mods().clone();
		for requirement in requirements {
			let captures = if let Some(captures) = REQUIREMENT_REGEX.captures(requirement) {
				captures
			}
			else {
				return Box::new(future::err(format!(r#"Could not parse requirement "{}""#, requirement).into()));
			};
			let name = ::factorio_mods_common::ModName::new(captures[1].to_string());
			let requirement_string = captures.get(2).map_or("*", |m| m.as_str());
			let requirement = match ::semver::VersionReq::parse(requirement_string) {
				Ok(requirement) => requirement,
				Err(err) => return Box::new(Err(err).chain_err(|| format!(r#"Could not parse "{}" as a valid version requirement"#, requirement_string)).into_future()),
			};

			reqs.insert(name, ::factorio_mods_common::ModVersionReq::new(requirement));
		}

		Box::new(
			::solve::compute_and_apply_diff(local_api, web_api, reqs.clone())
			.and_then(|result| Ok(if result {
				let config = config.with_mods(reqs);
				config.save()?
			})))
	}
}
