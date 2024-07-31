#[derive(clap::Args)]
pub(crate) struct SubCommand {
	#[arg(help = "requirements to install", required = true)]
	requirements: Vec<Requirement>,
}

#[derive(Clone, Debug)]
struct Requirement {
	name: factorio_mods_common::ModName,
	version: factorio_mods_common::ModVersionReq,
}

impl std::str::FromStr for Requirement {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use anyhow::Context;

		static REQUIREMENT_REGEX: std::sync::LazyLock<regex::Regex> =
			std::sync::LazyLock::new(|| regex::Regex::new(r"^([^@]+)(?:@(.*))?").unwrap());

		let captures = REQUIREMENT_REGEX.captures(s).with_context(|| format!(r#"Could not parse requirement "{s}""#))?;
		let name = factorio_mods_common::ModName(captures[1].to_owned());
		let version_string = captures.get(2).map_or("*", |m| m.as_str());
		let version =
			version_string.parse::<semver::VersionReq>()
			.with_context(|| format!(r#"could not parse "{version_string}" as a valid version requirement"#))?;
		let version = factorio_mods_common::ModVersionReq(version);

		Ok(Requirement {
			name,
			version,
		})
	}
}

impl SubCommand {
	pub(crate) async fn run(
		self,
		local_api: &factorio_mods_local::Api,
		web_api: &factorio_mods_web::Api,
		mut config: crate::config::Config,
		prompt_override: Option<bool>,
	) -> anyhow::Result<()> {
		let mods = config.mods.as_mut().unwrap();
		for requirement in self.requirements {
			mods.insert(requirement.name, requirement.version);
		}

		crate::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override).await?;

		Ok(())
	}
}
