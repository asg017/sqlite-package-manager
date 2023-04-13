mod http;
mod spm;

use crate::spm::Project;

use anyhow::{anyhow, Context, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};

fn command() -> Command {
    Command::new("sqlite-package-manager")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Alex Garcia")
        .about("The missing package manager for SQLite extensions and sqlite3.")
        .allow_missing_positional(true)
        .arg(
            Arg::new("prefix")
                .long("prefix")
                .help("Run spm commands in a different directory")
                .global(true),
        )
        .subcommand(
            Command::new("init")
                // https://docs.npmjs.com/cli/v8/commands/npm-init#synopsis
                .aliases(["create", "innit"])
                .about("Initialize a spm project"),
        )
        .subcommand(
            Command::new("add")
                .about("Add a SQLite extension to your spm project.")
                .arg(Arg::new("url").required(true))
                .arg(Arg::new("artifacts").required(false)),
        )
        .subcommand(
            Command::new("install")
                .aliases(
                    // https://docs.npmjs.com/cli/v8/commands/npm-install#synopsis
                    [
                        "i", "in", "ins", "inst", "insta", "instal", "isnt", "isnta", "isntal",
                        "isntall",
                    ],
                )
                .about("Install all listed SQLite extensions"),
        )
        .subcommand(
            Command::new("ci")
                .aliases(
                    // https://docs.npmjs.com/cli/v8/commands/npm-ci#synopsis
                    ["clean-install", "ic", "install-clean", "isntall-clean"],
                )
                .about("Install all listed SQLite extensions"),
        )
        .subcommand(
            Command::new("run")
                .about("Runs a command with pre-configured SQLite extenion path")
                .arg(Arg::new("command").action(ArgAction::Set).num_args(1..)),
        )
        .subcommand(
            Command::new("activate")
                .about("Activate a spm project to your shell. Use with command substitution."),
        )
        .subcommand(
            Command::new("deactivate")
                .about("Deactivate a spm project to your shell. Use with command substitution."),
        )
}

fn execute_matches(matches: ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("init", matches)) => {
            // TODO after spm.toml lookup traversal is added, change this to only use --prefix or CWD
            // bc no spm.toml will be available??
            let project = Project::resolve_from_args(matches)?;
            project.command_init()
        }
        Some(("add", matches)) => {
            let url = matches
                .get_one::<String>("url")
                .context("url is a required argument")?;
            let artifacts: Option<Vec<String>> = matches
                .get_many::<String>("artifacts")
                .map(|v| v.into_iter().map(|v| v.to_string()).collect());

            let project = Project::resolve_from_args(matches)?;
            project.command_add(url, artifacts)
        }
        Some(("install", matches)) => {
            let project = Project::resolve_from_args(matches)?;
            project.command_install()
        }
        Some(("ci", matches)) => {
            let project = Project::resolve_from_args(matches)?;
            project.command_clean_install()
        }
        Some(("activate", matches)) => {
            let project = Project::resolve_from_args(matches)?;
            project.command_activate()
        }
        Some(("deactivate", matches)) => {
            let project = Project::resolve_from_args(matches)?;
            project.command_deactivate()
        }
        Some(("run", matches)) => {
            let project = Project::resolve_from_args(matches)?;
            let command = matches
                .get_many::<String>("command")
                .context("command arguments required")?
                .collect::<Vec<_>>();
            let (program, arguments) = command
                .split_first()
                .ok_or_else(|| anyhow!("at least one argument is required"))?;
            project.command_run(program, arguments)
        }
        _ => Err(anyhow!("unknown subcommand")),
    }
}
fn main() {
    let matches = command().get_matches();
    let result = execute_matches(matches);
    if result.is_err() {
        println!("{:?}", result);
        std::process::exit(1);
    }
}
