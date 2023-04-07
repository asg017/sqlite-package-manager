use std::process::Stdio;

use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;

use crate::package_management::resolve_project_directory;

pub fn activate_command(matches: &ArgMatches) -> Result<()> {
    let project_directory = resolve_project_directory(matches)?;
    println!(
        "export DYLD_LIBRARY_PATH={}",
        project_directory.sqlite_extensions_path().to_string_lossy()
    );
    Ok(())
}
pub fn deactivate_command(_matches: &ArgMatches) -> Result<()> {
    println!("unset DYLD_LIBRARY_PATH");
    Ok(())
}

pub fn run_command(matches: &ArgMatches) -> Result<()> {
    let project_directory = resolve_project_directory(matches)?;
    let command = matches
        .get_many::<String>("command")
        .context("command arguments required")?
        .collect::<Vec<_>>();
    let (program, arguments) = command.split_first().ok_or_else(|| anyhow!("asdf"))?;
    let mut cmd = std::process::Command::new(program)
        .args(arguments)
        .env(
            "DYLD_LIBRARY_PATH",
            project_directory.sqlite_extensions_path().as_os_str(),
        )
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let status = cmd.wait()?;
    std::process::exit(status.code().map_or(1, |code| code));
}
