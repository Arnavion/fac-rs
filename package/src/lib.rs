//! Solves the given set of packages and requirements to produce a solution of packages to be installed.

#![deny(rust_2018_idioms, warnings)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::cognitive_complexity,
	clippy::default_trait_access,
	clippy::indexing_slicing,
	clippy::similar_names,
	clippy::too_many_lines,
	clippy::type_complexity,
	clippy::use_self,
)]

pub trait Package {
	type Name;
	type Version;
	type Dependency: Dependency<Name = Self::Name>;

	fn name(&self) -> &Self::Name;
	fn version(&self) -> &Self::Version;
	fn dependencies(&self) -> &[Self::Dependency];
}

pub trait Dependency {
	type Name;
	type Version;

	fn name(&self) -> &Self::Name;
	fn version(&self) -> &Self::Version;
	fn kind(&self) -> DependencyKind;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DependencyKind {
	Conflicts,
	Optional,
	Required,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Relation {
	Requires,
	Conflicts,
}

#[derive(Debug)]
pub struct Error<Name, Version> where
	Name: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
	Version: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static
{
	kind: ErrorKind<Name, Version>,
	backtrace: failure::Backtrace,
}

impl<Name, Version> Error<Name, Version> where
	Name: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
	Version: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
	/// Gets the kind of error
	pub fn kind(&self) -> &ErrorKind<Name, Version> {
		&self.kind
	}
}

impl<Name, Version> failure::Fail for Error<Name, Version> where
	Name: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
	Version: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
	fn cause(&self) -> Option<&dyn failure::Fail> {
		self.kind.cause()
	}

	fn backtrace(&self) -> Option<&failure::Backtrace> {
		Some(&self.backtrace)
	}
}

impl<Name, Version> std::fmt::Display for Error<Name, Version> where
	Name: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
	Version: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.kind.fmt(f)
	}
}

impl<Name, Version> From<ErrorKind<Name, Version>> for Error<Name, Version> where
	Name: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
	Version: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
	fn from(kind: ErrorKind<Name, Version>) -> Self {
		Error {
			kind,
			backtrace: Default::default(),
		}
	}
}

#[derive(Debug, failure_derive::Fail)]
pub enum ErrorKind<Name, Version> where
	Name: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
	Version: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
	#[fail(display = "{} {} both requires and conflicts with {} {}", package_name, package_version, dep_name, dep_version)]
	BothRequiresAndConflicts {
		package_name: Name,
		package_version: Version,
		dep_name: Name,
		dep_version: Version,
	},

	#[fail(display = "No packages found for {} that meet the specified requirements", _0)]
	NoPackagesMeetRequirements(Name),
}

/// A type alias for [`std::result::Result`]
pub type Result<Name, Version, T> = std::result::Result<T, Error<Name, Version>>;

pub fn compute_solution<I>(
	packages: I,
	reqs: &std::collections::HashMap<<<I as IntoIterator>::Item as Package>::Name, <<<I as IntoIterator>::Item as Package>::Dependency as Dependency>::Version>,
) -> Result<
	<<I as IntoIterator>::Item as Package>::Name,
	<<I as IntoIterator>::Item as Package>::Version,
	Option<std::collections::HashMap<<<I as IntoIterator>::Item as Package>::Name, <I as IntoIterator>::Item>>,
> where
	I: IntoIterator,
	<I as IntoIterator>::Item: Package + Clone,
	<<I as IntoIterator>::Item as Package>::Name: Clone + std::fmt::Debug + std::fmt::Display + Eq + std::hash::Hash + Send + Sync + 'static,
	<<I as IntoIterator>::Item as Package>::Version: AsRef<semver::Version> + Clone + std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
	<<<I as IntoIterator>::Item as Package>::Dependency as Dependency>::Version: AsRef<semver::VersionReq>,
{
	let mut graph: petgraph::Graph<_, Relation> =
		petgraph::data::FromElements::from_elements(
			packages.into_iter()
			.map(|package| petgraph::data::Element::Node { weight: package }));

	let mut edges_to_add = vec![];

	for node_index1 in graph.node_indices() {
		let package1 = &graph[node_index1];

		for node_index2 in graph.node_indices() {
			if node_index1 == node_index2 {
				continue;
			}

			let package2 = &graph[node_index2];

			if package1.name() == package2.name() {
				edges_to_add.push((node_index1, node_index2, Relation::Conflicts));
			}
			else {
				let mut requires = false;
				let mut conflicts = false;

				for dep in package1.dependencies() {
					if dep.name() != package2.name() {
						continue;
					}

					match (dep.kind(), dep.version().as_ref().matches(package2.version().as_ref())) {
						(DependencyKind::Required, true) => requires = true,
						(DependencyKind::Conflicts, true) |
						(DependencyKind::Optional, false) => conflicts = true,
						_ => continue,
					}
				}

				match (requires, conflicts) {
					(true, true) => return Err((ErrorKind::BothRequiresAndConflicts {
						package_name: package1.name().clone(),
						package_version: package1.version().clone(),
						dep_name: package2.name().clone(),
						dep_version: package2.version().clone(),
					}).into()),
					(true, false) => edges_to_add.push((node_index1, node_index2, Relation::Requires)),
					(false, true) => edges_to_add.push((node_index1, node_index2, Relation::Conflicts)),
					(false, false) => (),
				}
			}
		}
	}

	for edge_to_add in edges_to_add {
		assert!(graph.find_edge(edge_to_add.0, edge_to_add.1).is_none());

		graph.add_edge(edge_to_add.0, edge_to_add.1, edge_to_add.2);
	}

	loop {
		let mut node_indices_to_remove = std::collections::HashSet::new();

		{
			let mut name_to_node_indices: std::collections::HashMap<_, Vec<petgraph::graph::NodeIndex>> = Default::default();
			for node_index in graph.node_indices() {
				let package = &graph[node_index];
				name_to_node_indices.entry(package.name()).or_default().push(node_index);
			}

			for name in reqs.keys() {
				match name_to_node_indices.get(name) {
					Some(node_indices) if !node_indices.is_empty() => (),
					_ => return Err(ErrorKind::NoPackagesMeetRequirements(name.clone()).into()),
				}
			}

			node_indices_to_remove.extend(graph.node_indices().filter(|&node_index| {
				let package = &graph[node_index];

				let keep = match reqs.get(package.name()) {
					// Required package
					Some(req) => req.as_ref().matches(package.version().as_ref()),

					// Required by another package
					None => graph.edges_directed(node_index, petgraph::Direction::Incoming).any(|edge|
						if let Relation::Requires = *edge.weight() {
							true
						}
						else {
							false
						}),
				};

				// All required dependencies satisfied
				let keep = keep &&
					package.dependencies().iter()
					.filter(|dep| dep.kind() == DependencyKind::Required)
					.all(|dep|
						name_to_node_indices.get(dep.name())
						.map_or(false, |dep_node_indices|
							dep_node_indices.iter()
							.any(|&dep_node_index| dep.version().as_ref().matches(graph[dep_node_index].version().as_ref()))));

				!keep
			}));

			if node_indices_to_remove.is_empty() {
				for node_indices in name_to_node_indices.values() {
					for &node_index1 in node_indices {
						let package1 = &graph[node_index1];

						let neighbors1: std::collections::HashSet<_> =
							graph.edges_directed(node_index1, petgraph::Direction::Incoming)
							.map(|edge| (petgraph::Direction::Incoming, edge.weight(), petgraph::visit::EdgeRef::source(&edge)))
							.chain(
								graph.edges(node_index1)
								.map(|edge| (petgraph::Direction::Outgoing, edge.weight(), petgraph::visit::EdgeRef::target(&edge))))
							.filter(|&(_, _, neighbor_node_index)| graph[neighbor_node_index].name() != package1.name())
							.collect();

						for &node_index2 in node_indices {
							if node_index2 > node_index1 {
								let package2 = &graph[node_index2];

								let neighbors2: std::collections::HashSet<_> =
									graph.edges_directed(node_index2, petgraph::Direction::Incoming)
									.map(|edge| (petgraph::Direction::Incoming, edge.weight(), petgraph::visit::EdgeRef::source(&edge)))
									.chain(
										graph.edges(node_index2)
										.map(|edge| (petgraph::Direction::Outgoing, edge.weight(), petgraph::visit::EdgeRef::target(&edge))))
									.filter(|&(_, _, neighbor_node_index)| graph[neighbor_node_index].name() != package2.name())
									.collect();

								if neighbors1 == neighbors2 {
									// Two packages with identical requirements and conflicts. Remove the one with the lower version.
									if package1.version().as_ref() < package2.version().as_ref() {
										node_indices_to_remove.insert(node_index1);
									}
									else {
										node_indices_to_remove.insert(node_index2);
									}
								}
							}
						}
					}
				}
			}

			if node_indices_to_remove.is_empty() {
				for req in reqs.keys() {
					let node_indices = name_to_node_indices.get(req).unwrap();

					let mut common_conflicts = None;

					for &node_index in node_indices {
						let conflicts: std::collections::HashSet<_> =
							graph.edges(node_index)
							.filter_map(|edge|
								if let Relation::Conflicts = *edge.weight() {
									Some(petgraph::visit::EdgeRef::target(&edge))
								}
								else {
									None
								})
							.collect();

						common_conflicts = if let Some(existing) = common_conflicts {
							Some(&existing & &conflicts)
						}
						else {
							Some(conflicts)
						};
					}

					if let Some(common_conflicts) = common_conflicts {
						node_indices_to_remove.extend(common_conflicts);
					}
				}
			}
		}

		if node_indices_to_remove.is_empty() {
			break;
		}
		else {
			let node_indices_to_remove = itertools::Itertools::sorted_by(node_indices_to_remove.into_iter(), |i1, i2| i1.cmp(i2).reverse());

			for node_index in node_indices_to_remove {
				graph.remove_node(node_index);
			}
		}
	}

	let possibilities: Vec<_> = {
		let mut name_to_packages: std::collections::HashMap<_, Vec<_>> = Default::default();
		for node in graph.into_nodes_edges().0 {
			let package = node.weight;
			name_to_packages.entry(package.name().clone()).or_default().push(Some(package));
		}

		name_to_packages.into_iter().map(|(name, mut packages)| {
			if !reqs.contains_key(&name) {
				packages.insert(0, None);
			}

			packages
		}).collect()
	};

	let possibilities: Vec<Vec<_>> = possibilities.iter().map(|p| p.iter().map(Option::as_ref).collect()).collect();
	let possibilities: Vec<_> = possibilities.iter().map(AsRef::as_ref).collect();
	let mut permutater = Permutater::new(&possibilities[..]);

	let mut values = vec![None; possibilities.len()];

	let mut best_solution = None;

	while permutater.next(&mut values) {
		let solution = values.iter().filter_map(|package| package.map(|package| (package.name(), package))).collect();

		if is_valid(&solution) {
			best_solution = std::cmp::max(best_solution, Some(Solution(solution)));
		}
	}

	Ok(best_solution.map(|best_solution| best_solution.0.into_iter().map(|(name, package)| (name.clone(), package.clone())).collect()))
}

fn is_valid<P>(solution: &std::collections::HashMap<&<P as Package>::Name, &P>) -> bool where
	P: Package,
	<P as Package>::Name: Eq + std::hash::Hash,
	<P as Package>::Version: AsRef<semver::Version>,
	<<P as Package>::Dependency as Dependency>::Version: AsRef<semver::VersionReq>,
{
	for package in solution.values() {
		for dep in package.dependencies() {
			if let Some(package) = solution.get(dep.name()) {
				if !dep.version().as_ref().matches(package.version().as_ref()) {
					return false;
				}
			}
			else if dep.kind() == DependencyKind::Required {
				return false;
			}
		}
	}

	true
}

struct Solution<'a, P>(std::collections::HashMap<&'a <P as Package>::Name, &'a P>) where
	P: Package,
	<P as Package>::Name: Eq + std::hash::Hash,
;

impl<P> Ord for Solution<'_, P> where
	P: Package,
	<P as Package>::Name: Eq + std::hash::Hash,
	<P as Package>::Version: AsRef<semver::Version>,
{
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		for (n1, i1) in &self.0 {
			if let Some(i2) = other.0.get(n1) {
				match i1.version().as_ref().cmp(i2.version().as_ref()) {
					std::cmp::Ordering::Equal => (),
					o => return o,
				}
			}
		}

		self.0.len().cmp(&other.0.len()).reverse()
	}
}

impl<P> PartialOrd for Solution<'_, P> where
	P: Package,
	<P as Package>::Name: Eq + std::hash::Hash,
	<P as Package>::Version: AsRef<semver::Version>,
{
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<P> PartialEq for Solution<'_, P> where
	P: Package,
	<P as Package>::Name: Eq + std::hash::Hash,
	<P as Package>::Version: AsRef<semver::Version>,
{
	fn eq(&self, other: &Self) -> bool {
		self.cmp(other) == std::cmp::Ordering::Equal
	}
}

impl<P> Eq for Solution<'_, P>
 where
	P: Package,
	<P as Package>::Name: Eq + std::hash::Hash,
	<P as Package>::Version: AsRef<semver::Version>,
{
}

struct Permutater<'a, T> {
	state: Vec<usize>,
	possibilities: &'a [&'a [T]],
	run_once: bool,
}

impl<'a, T> Permutater<'a, T> where T: Copy {
	fn new(possibilities: &'a [&'a [T]]) -> Permutater<'a, T> {
		Permutater {
			state: vec![0; possibilities.len()],
			possibilities,
			run_once: false,
		}
	}

	fn next(&mut self, values: &mut [T]) -> bool {
		assert_eq!(self.possibilities.len(), values.len());

		if self.advance(0) {
			for (value_index, &element_index) in self.state.iter().enumerate() {
				values[value_index] = self.possibilities[value_index][element_index];
			}

			true
		}
		else {
			false
		}
	}

	fn advance(&mut self, index: usize) -> bool {
		if index >= self.possibilities.len() {
			return false;
		}

		if self.run_once {
			if self.state[index] < self.possibilities[index].len() - 1 {
				self.state[index] += 1;
				true
			}
			else {
				self.state[index] = 0;
				self.advance(index + 1)
			}
		}
		else {
			self.run_once = true;
			true
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_permutater() {
		let possibilities = vec![vec![None, Some("a"), Some("b")], vec![None, Some("c")]];
		let possibilities: Vec<_> = possibilities.iter().map(AsRef::as_ref).collect();
		let mut values = vec![None; possibilities.len()];

		let mut permutater = Permutater::new(&possibilities);

		assert!(permutater.next(&mut values));
		assert_eq!(values, [None, None]);

		assert!(permutater.next(&mut values));
		assert_eq!(values, [Some("a"), None]);

		assert!(permutater.next(&mut values));
		assert_eq!(values, [Some("b"), None]);

		assert!(permutater.next(&mut values));
		assert_eq!(values, [None, Some("c")]);

		assert!(permutater.next(&mut values));
		assert_eq!(values, [Some("a"), Some("c")]);

		assert!(permutater.next(&mut values));
		assert_eq!(values, [Some("b"), Some("c")]);

		assert!(!permutater.next(&mut values));
	}
}
