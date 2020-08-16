#[derive(Debug, structopt::StructOpt)]
pub(crate) struct SubCommand {
	#[structopt(help = "requirements to install", required = true)]
	requirements: Vec<Requirement>,
}

#[derive(Debug)]
struct Requirement {
	name: factorio_mods_common::ModName,
	version: factorio_mods_common::ModVersionReq,
}

impl std::str::FromStr for Requirement {
	type Err = crate::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use crate::ResultExt;

		static REQUIREMENT_REGEX: once_cell::sync::Lazy<regex::Regex> =
			once_cell::sync::Lazy::new(|| regex::Regex::new(r"^([^@]+)(?:@(.*))?").unwrap());

		let captures = match REQUIREMENT_REGEX.captures(s) {
			Some(captures) => captures,
			None => return Err(format!(r#"Could not parse requirement "{}""#, s).into()),
		};
		let name = factorio_mods_common::ModName(captures[1].to_string());
		let version_string = captures.get(2).map_or("*", |m| m.as_str());
		let version =
			version_string.parse::<semver::VersionReq>()
			.with_context(|| format!(r#"could not parse "{}" as a valid version requirement"#, version_string))?;
		let version = factorio_mods_common::ModVersionReq(version);

		Ok(Requirement {
			name,
			version,
		})
	}
}

impl SubCommand {
	pub(crate) async fn run<'a>(
		self,
		local_api: Result<&'a factorio_mods_local::API, crate::Error>,
		web_api: Result<&'a factorio_mods_web::API, crate::Error>,
		mut config: crate::config::Config,
		prompt_override: Option<bool>,
	) -> Result<(), crate::Error> {
		let local_api = local_api?;
		let web_api = web_api?;

		let mods = config.mods.as_mut().unwrap();
		for requirement in self.requirements {
			mods.insert(requirement.name, requirement.version);
		}

		crate::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override).await?;

		Ok(())
	}
}
