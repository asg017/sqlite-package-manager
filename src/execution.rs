use std::{ffi::OsString, process::Stdio};

use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;

use crate::{package_management::resolve_project_directory, spm::ProjectDirectory};

#[cfg(target_os = "linux")]
const LIBRARY_PATH_ENV_VAR: &str = "LD_LIBRARY_PATH";
#[cfg(target_os = "macos")]
const LIBRARY_PATH_ENV_VAR: &str = "DYLD_LIBRARY_PATH";
#[cfg(target_os = "windows")]
const LIBRARY_PATH_ENV_VAR: &str = "PATH";

fn get_library_path_with_project(project: &ProjectDirectory) -> Result<OsString> {
    match std::env::var_os(LIBRARY_PATH_ENV_VAR) {
        Some(paths) => {
            let mut paths = std::env::split_paths(&paths).collect::<Vec<_>>();
            paths.push(project.sqlite_extensions_path());
            Ok(std::env::join_paths(paths)
                .context("Invalid path, is there a semicolor ':' somewhere in a path?")?)
        }
        None => Ok(project.sqlite_extensions_path().into()),
    }
}
pub fn activate_command(matches: &ArgMatches) -> Result<()> {
    let project = resolve_project_directory(matches)?;
    let library_path = get_library_path_with_project(&project)?;
    println!(
        "export {}={}",
        LIBRARY_PATH_ENV_VAR,
        library_path.to_string_lossy()
    );
    Ok(())
}
pub fn deactivate_command(_matches: &ArgMatches) -> Result<()> {
    // TODO
    println!("unset {}", LIBRARY_PATH_ENV_VAR);
    Ok(())
}

pub fn run_command(matches: &ArgMatches) -> Result<()> {
    let project = resolve_project_directory(matches)?;
    let library_path = get_library_path_with_project(&project)?;

    let command = matches
        .get_many::<String>("command")
        .context("command arguments required")?
        .collect::<Vec<_>>();
    let (program, arguments) = command
        .split_first()
        .ok_or_else(|| anyhow!("at least one argument is required"))?;
    let mut cmd = std::process::Command::new(program)
        .args(arguments)
        .env(LIBRARY_PATH_ENV_VAR, library_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let status = cmd.wait()?;
    std::process::exit(status.code().map_or(1, |code| code));
}
