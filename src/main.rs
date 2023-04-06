mod execution;
mod package_management;
mod spm;

use crate::{execution::*, package_management::*};

use anyhow::anyhow;
use clap::{Arg, ArgAction, Command};

fn main() {
    let matches = Command::new("sqlite-package-manager")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Alex Garcia")
        .about("The missing package manager for SQLite extensions and sqlite3.")
        .allow_missing_positional(true)
        .subcommand(
            Command::new("run")
                .about("Runs a command with pre-configured SQLite extenion path")
                .arg(Arg::new("please").action(ArgAction::Set).num_args(1..)),
        )
        .subcommand(
            Command::new("activate")
                .about("Activate a spm project to your shell. Use with command substitution."),
        )
        .subcommand(
            Command::new("deactivate")
                .about("Deactivate a spm project to your shell. Use with command substitution."),
        )
        .subcommand(Command::new("init").about("Initialize a spm project"))
        .subcommand(
            Command::new("add")
                .about("Add a SQLite extension to your spm project.")
                .arg(Arg::new("url").required(true))
                .arg(Arg::new("artifact").required(false)),
        )
        .subcommand(Command::new("generate").about("gen"))
        .subcommand(Command::new("install").about("Install a SQLite extension"))
        .get_matches();
    let result = match matches.subcommand() {
        Some(("init", _)) => init_command(),
        Some(("activate", _)) => activate_command(),
        Some(("deactivate", _)) => deactivate_command(),
        Some(("run", matches)) => run_command(matches),
        Some(("add", matches)) => add_command(matches),
        Some(("generate", _)) => generate_command(),
        Some(("install", matches)) => install_command(matches),
        _ => Err(anyhow!("asdf")),
    };
    match result {
        Ok(()) => (),
        Err(_) => {
            std::process::exit(1);
        }
    }
}
