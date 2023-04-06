use std::process::Stdio;

use anyhow::Result;
use clap::ArgMatches;

pub fn activate_command() -> Result<()> {
    println!(
        "export DYLD_LIBRARY_PATH={}",
        std::env::current_dir()
            .unwrap()
            .join("sqlite_extensions")
            .to_string_lossy()
    );
    Ok(())
}
pub fn deactivate_command() -> Result<()> {
    println!("unset DYLD_LIBRARY_PATH");
    Ok(())
}

pub fn run_command(matches: &ArgMatches) -> Result<()> {
    let mut x = matches
        .get_many::<String>("please")
        .unwrap()
        .collect::<Vec<_>>();

    let mut cmd = std::process::Command::new(x[0])
        .args(&mut x[1..])
        .env(
            "DYLD_LIBRARY_PATH",
            std::env::current_dir()
                .unwrap()
                .join("sqlite_extensions")
                .as_os_str(),
        )
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        //.output()
        .expect("failed to execute process");

    let status = cmd.wait();
    std::process::exit(status.unwrap().code().unwrap());
}
