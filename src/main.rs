//! A CLI tool to manage Factorio mods.

use anyhow::Context;

mod enable_disable;
mod install;
mod list;
mod uninstall;
mod search;
mod show;
mod update;

mod config;
mod solve;
mod util;

#[derive(clap::Parser)]
#[command(about, author)]
pub(crate) struct Options {
	#[arg(help = "Path to fac config file. Defaults to .../fac/config.json", short = 'c', value_parser)]
	config: Option<std::path::PathBuf>,

	#[arg(help = "Answer yes to all prompts", short = 'y')]
	yes: bool,

	#[arg(help = "Answer no to all prompts", short = 'n', conflicts_with = "yes")]
	no: bool,

	#[command(subcommand)]
	subcommand: SubCommand,
}

#[derive(clap::Subcommand)]
pub(crate) enum SubCommand {
	#[command(name = "disable", about = "Disable mods")]
	Disable(enable_disable::DisableSubCommand),

	#[command(name = "enable", about = "Enable mods")]
	Enable(enable_disable::EnableSubCommand),

	#[command(name = "install", about = "Install (or update) mods", visible_alias = "add")]
	Install(install::SubCommand),

	#[command(name = "list", about = "List installed mods and their status")]
	List(list::SubCommand),

	#[command(name = "search", about = "Search the mods database")]
	Search(search::SubCommand),

	#[command(name = "show", about = "Show details about specific mods")]
	Show(show::SubCommand),

	#[command(name = "uninstall", about = "Uninstall mods", visible_alias = "remove")]
	Uninstall(uninstall::SubCommand),

	#[command(name = "update", about = "Update installed mods")]
	Update(update::SubCommand),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
	std::env::set_var("RUST_BACKTRACE", "1");

	let options: Options = clap::Parser::parse();

	let prompt_override = match (options.yes, options.no) {
		(true, false) => Some(true),
		(false, true) => Some(false),
		(false, false) => None,
		(true, true) => unreachable!(),
	};

	let mut config = crate::config::Config::load(options.config)?;

	let local_api: anyhow::Result<_> = match (&config.install_directory, &config.user_directory) {
		(Some(install_directory), Some(user_directory)) =>
			factorio_mods_local::Api::new(install_directory, user_directory)
			.context("could not initialize local API"),

		(None, _) =>
			Err(anyhow::Error::new(factorio_mods_local::Error::InstallDirectoryNotFound))
			.context(r#"could not initialize local API. Consider setting "install_directory" to the path in the config file."#),

		(_, None) =>
			Err(anyhow::Error::new(factorio_mods_local::Error::UserDirectoryNotFound))
			.context(r#"could not initialize local API. Consider setting "user_directory" to the path in the config file."#),
	};

	if config.mods.is_none() {
		if let Ok(local_api) = &local_api {
			// Default mods list is the list of all currently installed mods with a * requirement
			let installed_mods =
				itertools::Itertools::try_collect::<_, _, _>(
					local_api.installed_mods().context("could not enumerate installed mods")?
					.map(|mod_|
						mod_
						.map(|mod_| (mod_.info.name, factorio_mods_common::ModVersionReq(semver::VersionReq::STAR)))
						.context("could not process an installed mod")))
				.context("could not enumerate installed mods")?;
			config.mods = Some(installed_mods);
		}
	}

	let web_api = factorio_mods_web::Api::new().context("could not initialize web API");


	match options.subcommand {
		SubCommand::Disable(parameters) => parameters.run(
			&local_api?,
			prompt_override,
		)?,

		SubCommand::Enable(parameters) => parameters.run(
			&local_api?,
			prompt_override,
		)?,

		SubCommand::Install(parameters) => parameters.run(
			&local_api?,
			&web_api?,
			config,
			prompt_override,
		).await?,

		SubCommand::List(_) => list::SubCommand::run(
			&local_api?,
		)?,

		SubCommand::Search(parameters) => parameters.run(
			&web_api?,
		).await?,

		SubCommand::Show(parameters) => parameters.run(
			&web_api?,
		).await?,

		SubCommand::Uninstall(parameters) => parameters.run(
			&local_api?,
			&web_api?,
			config,
			prompt_override,
		).await?,

		SubCommand::Update(parameters) => parameters.run(
			&local_api?,
			&web_api?,
			config,
			prompt_override,
		).await?,
	}

	Ok(())
}

fn textwrap_options() -> textwrap::Options<'static> {
	textwrap::Options::with_termwidth()
		.initial_indent("    ")
		.subsequent_indent("    ")
		.break_words(true)
		.wrap_algorithm(textwrap::WrapAlgorithm::OptimalFit(Default::default()))
		.word_separator(textwrap::WordSeparator::UnicodeBreakProperties)
		.word_splitter(textwrap::WordSplitter::NoHyphenation)
}
