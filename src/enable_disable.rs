use ::ResultExt;

pub struct EnableSubCommand;

impl ::util::SubCommand for EnableSubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Enable mods.")
			(@arg mods: ... +required index(1) "mods to enable"))
	}

	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, local_api: ::Result<::factorio_mods_local::API>, _: ::Result<::factorio_mods_web::API>) -> ::Result<()> {
		enable_disable(matches, local_api, true)
	}
}

pub struct DisableSubCommand;

impl ::util::SubCommand for DisableSubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Disable mods.")
			(@arg mods: ... +required index(1) "mods to disable"))
	}

	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, local_api: ::Result<::factorio_mods_local::API>, _: ::Result<::factorio_mods_web::API>) -> ::Result<()> {
		enable_disable(matches, local_api, false)
	}
}

fn enable_disable<'a>(matches: &::clap::ArgMatches<'a>, local_api: ::Result<::factorio_mods_local::API>, enable: bool) -> ::Result<()> {
	let local_api = local_api?;

	let mods = matches.values_of("mods").unwrap();

	let all_installed_mods: ::Result<::multimap::MultiMap<_, _>> =
		local_api.installed_mods().chain_err(|| "Could not enumerate installed mods")?
		.map(|mod_| mod_.map(|mod_| (mod_.info().name().clone(), mod_)).chain_err(|| "Could not process an installed mod"))
		.collect();
	let all_installed_mods = all_installed_mods.chain_err(|| "Could not enumerate installed mods")?;

	for (name, installed_mods) in &all_installed_mods {
		if installed_mods.len() > 1 {
			bail!("There is more than one version of {} installed. Run `fac update` or remove all but one version manually.", name);
		}
	}

	let all_installed_mods: ::std::collections::HashMap<_, _> =
		all_installed_mods.into_iter().map(|(name, mut installed_mods)| (name, installed_mods.remove(0))).collect();

	let mut to_enable = Default::default();

	for name in mods {
		visit_installed_mod(&::factorio_mods_common::ModName::new(name.to_string()), &mut to_enable, &all_installed_mods)?
	}

	let mut to_enable: Vec<_> = to_enable.into_iter().map(|(_, installed_mod)| installed_mod).collect();
	to_enable.sort_by(|mod1, mod2| mod1.info().name().cmp(mod2.info().name()));

	println!("The following mods will be {}:", if enable { "enabled" } else { "disabled" });
	for to_enable in &to_enable {
		println!("{}", to_enable.info().name());
	}

	println!();
	if !::util::prompt_continue()? {
		return Ok(());
	}

	local_api.set_enabled(to_enable, enable).chain_err(|| format!("Could not {} mods", if enable { "enable" } else { "disable" }))
}

fn visit_installed_mod<'a>(
	name: &::factorio_mods_common::ModName,
	to_enable: &mut ::std::collections::HashMap<&'a ::factorio_mods_common::ModName, &'a ::factorio_mods_local::InstalledMod>,
	all_installed_mods: &'a ::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_local::InstalledMod>,
) -> ::Result<()> {
	if let Some(installed_mod) = all_installed_mods.get(name) {
		if to_enable.insert(installed_mod.info().name(), installed_mod).is_none() {
			for dep in installed_mod.info().dependencies() {
				if *dep.required() && &**dep.name() != "base" {
					visit_installed_mod(dep.name(), to_enable, all_installed_mods)?
				}
			}
		}
	}
	else {
		bail!("Mod {} is a required dependency but not installed. Run `fac update` to install missing dependencies.", name)
	}

	Ok(())
}
