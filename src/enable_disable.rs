#[derive(Debug, structopt::StructOpt)]
pub(crate) struct EnableSubCommand {
	#[structopt(help = "mods to enable", required = true)]
	names: Vec<factorio_mods_common::ModName>,
}

impl EnableSubCommand {
	pub(crate) async fn run(
		self,
		local_api: &factorio_mods_local::Api,
		prompt_override: Option<bool>,
	) -> Result<(), crate::Error> {
		enable_disable(self.names, local_api, prompt_override, true).await?;
		Ok(())
	}
}

#[derive(Debug, structopt::StructOpt)]
pub(crate) struct DisableSubCommand {
	#[structopt(help = "mods to disable", required = true)]
	names: Vec<factorio_mods_common::ModName>,
}

impl DisableSubCommand {
	pub(crate) async fn run(
		self,
		local_api: &factorio_mods_local::Api,
		prompt_override: Option<bool>,
	) -> Result<(), crate::Error> {
		enable_disable(self.names, local_api, prompt_override, false).await?;
		Ok(())
	}
}

pub(crate) async fn enable_disable(
	mods: Vec<factorio_mods_common::ModName>,
	local_api: &factorio_mods_local::Api,
	prompt_override: Option<bool>,
	enable: bool,
) -> Result<(), crate::Error> {
	use crate::ResultExt;

	let mut all_installed_mods: std::collections::BTreeMap<_, Vec<_>> = Default::default();
	for mod_ in local_api.installed_mods().context("could not enumerate installed mods")? {
		let mod_ = mod_.context("could not process an installed mod")?;
		all_installed_mods.entry(mod_.info.name.clone()).or_default().push(mod_);
	}

	for (name, installed_mods) in &all_installed_mods {
		if installed_mods.len() > 1 {
			println!("There is more than one version of {} installed. Run `fac update` or remove all but one version manually.", name);
			return Ok(());
		}
	}

	let mut graph = petgraph::Graph::new();

	let name_to_node_index: std::collections::BTreeMap<_, _> =
		all_installed_mods.into_iter().map(|(name, mut installed_mods)| (name, graph.add_node(installed_mods.remove(0)))).collect();

	let mut edges_to_add = vec![];
	for node_index in graph.node_indices() {
		let installed_mod = &graph[node_index];
		for dep in &installed_mod.info.dependencies {
			if dep.kind == package::DependencyKind::Required && dep.name.0 != "base" {
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
					return Ok(());
				}
			}
		}
	}
	for edge_to_add in edges_to_add {
		graph.add_edge(edge_to_add.0, edge_to_add.1, ());
	}

	let mut to_change = std::collections::BTreeSet::new();

	for name in mods {
		if let Some(&node_index) = name_to_node_index.get(&name) {
			let bfs = petgraph::visit::Bfs::new(&graph, node_index);
			to_change.extend(petgraph::visit::Walker::iter(bfs, &graph));
		}
		else {
			println!("No match found for mod {}", name);
			return Ok(());
		}
	}

	let mut to_change: Vec<_> = to_change.into_iter().map(|node_index| &graph[node_index]).collect();
	to_change.sort_by(|mod1, mod2| mod1.info.name.cmp(&mod2.info.name));

	println!("The following mods will be {}:", if enable { "enabled" } else { "disabled" });
	for to_change in &to_change {
		println!("{}", to_change.info.name);
	}

	println!();
	if !crate::util::prompt_continue(prompt_override)? {
		return Ok(());
	}

	local_api.set_enabled(to_change, enable)
	.with_context(|| format!("could not {} mods", if enable { "enable" } else { "disable" }))?;

	Ok(())
}
