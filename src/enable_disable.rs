use ::futures::{ future, Future, IntoFuture };

use ::ResultExt;

pub struct EnableSubCommand;

impl ::util::SubCommand for EnableSubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Enable mods.")
			(@arg mods: ... +required index(1) "mods to enable"))
	}

	fn run<'a>(
		&'a self,
		matches: &'a ::clap::ArgMatches<'a>,
		local_api: ::Result<&'a ::factorio_mods_local::API>,
		_: ::Result<&'a ::factorio_mods_web::API>,
	) -> Box<Future<Item = (), Error = ::Error> + 'a> {
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

	fn run<'a>(
		&'a self,
		matches: &'a ::clap::ArgMatches<'a>,
		local_api: ::Result<&'a ::factorio_mods_local::API>,
		_: ::Result<&'a ::factorio_mods_web::API>,
	) -> Box<Future<Item = (), Error = ::Error> + 'a> {
		enable_disable(matches, local_api, false)
	}
}

fn enable_disable<'a>(
	matches: &'a ::clap::ArgMatches<'a>,
	local_api: ::Result<&'a ::factorio_mods_local::API>,
	enable: bool,
) -> Box<Future<Item = (), Error = ::Error> + 'a> {
	let result: ::Result<_> = do catch {
		let local_api = local_api?;

		let mods = matches.values_of("mods").unwrap();

		let all_installed_mods: ::Result<::multimap::MultiMap<_, _>> =
			local_api.installed_mods().chain_err(|| "Could not enumerate installed mods")?
			.map(|mod_| mod_.map(|mod_| (mod_.info.name.clone(), mod_)).chain_err(|| "Could not process an installed mod"))
			.collect();
		let all_installed_mods = all_installed_mods.chain_err(|| "Could not enumerate installed mods")?;

		for (name, installed_mods) in &all_installed_mods {
			if installed_mods.len() > 1 {
				println!("There is more than one version of {} installed. Run `fac update` or remove all but one version manually.", name);
				return Box::new(future::ok(()));
			}
		}

		let mut graph = ::petgraph::Graph::new();

		let name_to_node_index: ::std::collections::HashMap<_, _> =
			all_installed_mods.into_iter().map(|(name, mut installed_mods)| (name, graph.add_node(installed_mods.remove(0)))).collect();

		let mut edges_to_add = vec![];
		for node_index in graph.node_indices() {
			let installed_mod = &graph[node_index];
			for dep in &installed_mod.info.dependencies {
				if dep.required && dep.name.0 != "base" {
					if let Some(&dep_node_index) = name_to_node_index.get(&dep.name) {
						if enable {
							edges_to_add.push((node_index, dep_node_index));
						}
						else {
							edges_to_add.push((dep_node_index, node_index));
						}
					}
					else {
						println!("Mod {} is a required dependency of {} but isn't installed. Run `fac update` to install missing dependencies.", dep.name, installed_mod.info.name);
						return Box::new(future::ok(()));
					}
				}
			}
		}
		for edge_to_add in edges_to_add {
			graph.add_edge(edge_to_add.0, edge_to_add.1, ());
		}

		let mut to_change = ::std::collections::HashSet::new();

		for name in mods {
			if let Some(&node_index) = name_to_node_index.get(&::factorio_mods_common::ModName(name.to_string())) {
				let bfs = ::petgraph::visit::Bfs::new(&graph, node_index);
				to_change.extend(::petgraph::visit::Walker::iter(bfs, &graph));
			}
			else {
				println!("No match found for mod {}", name);
				return Box::new(future::ok(()));
			}
		}

		let mut to_change: Vec<_> = to_change.into_iter().map(|node_index| &graph[node_index]).collect();
		to_change.sort_by(|mod1, mod2| mod1.info.name.cmp(&mod2.info.name));

		println!("The following mods will be {}:", if enable { "enabled" } else { "disabled" });
		for to_change in &to_change {
			println!("{}", to_change.info.name);
		}

		println!();
		if !::util::prompt_continue()? {
			return Box::new(future::ok(()));
		}

		local_api.set_enabled(to_change, enable).chain_err(|| format!("Could not {} mods", if enable { "enable" } else { "disable" }))?;
	};
	Box::new(result.into_future())
}
