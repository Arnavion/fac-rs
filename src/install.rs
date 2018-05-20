use ::futures::Future;

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

	fn run<'a>(
		&'a self,
		matches: &'a ::clap::ArgMatches<'a>,
		local_api: ::Result<&'a ::factorio_mods_local::API>,
		web_api: ::Result<&'a ::factorio_mods_web::API>,
		prompt_override: Option<bool>,
	) -> Box<Future<Item = (), Error = ::Error> + 'a> {
		use ::ResultExt;

		Box::new(::async_block! {
			let requirements = matches.values_of("requirements").unwrap();

			let local_api = local_api?;
			let web_api = web_api?;

			let mut config = ::config::Config::load(local_api)?;

			for requirement in requirements {
				let captures = match REQUIREMENT_REGEX.captures(requirement) {
					Some(captures) => captures,
					None => bail!(r#"Could not parse requirement "{}""#, requirement),
				};
				let name = ::factorio_mods_common::ModName(captures[1].to_string());
				let requirement_string = captures.get(2).map_or("*", |m| m.as_str());
				let requirement = match requirement_string.parse() {
					Ok(requirement) => requirement,
					Err(err) => return Err(err).chain_err(|| format!(r#"Could not parse "{}" as a valid version requirement"#, requirement_string)),
				};

				config.mods.insert(name, ::factorio_mods_common::ModVersionReq(requirement));
			}

			::await!(::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override))?;

			Ok(())
		})
	}
}
