lazy_static! {
	static ref REQUIREMENT_REGEX: ::regex::Regex = ::regex::Regex::new(r"^([^@]+)(?:@(.*))?").unwrap();
}

pub struct SubCommand;

impl<FL, FW> ::util::SubCommand<FL, FW> for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Install (or update) mods.")
			(@arg requirements: ... +required index(1) "requirements to install"))
	}

	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, local_api: FL, web_api: FW)
		where FL: FnOnce() -> ::factorio_mods_local::API, FW: FnOnce() -> ::factorio_mods_web::API {
		let web_api = web_api();
		let local_api = local_api();

		let requirements = matches.values_of("requirements").unwrap();

		let config = ::config::Config::load(&local_api);
		let mut reqs = config.mods().clone();
		reqs.extend(requirements.into_iter().map(|requirement| {
			let captures = REQUIREMENT_REGEX.captures(requirement).unwrap();
			let name = ::factorio_mods_common::ModName::new(captures[1].to_string());
			let requirement_string = captures.get(2).map(|m| m.as_str()).unwrap_or("*");
			let requirement = ::semver::VersionReq::parse(requirement_string).unwrap();
			(name.clone(), ::factorio_mods_common::ModVersionReq::new(requirement))
		}));

		if ::solve::compute_and_apply_diff(&local_api, &web_api, &reqs) {
			let config = config.with_mods(reqs);
			config.save();
		}
	}
}
