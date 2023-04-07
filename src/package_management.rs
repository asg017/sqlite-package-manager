use std::collections::HashMap;

use crate::spm::{ProjectDirectory, SpmLock, SpmLockExtension, SpmPackageJson, SpmToml};

use clap::ArgMatches;

use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use std::io::BufReader;
use tar::Archive;

use toml_edit::{value, Document};

pub(crate) fn resolve_project_directory(matches: &ArgMatches) -> Result<ProjectDirectory> {
    match matches.get_one::<String>("prefix") {
        Some(base_directory) => Ok(ProjectDirectory::new(base_directory.into())),
        None => Ok(ProjectDirectory::new(std::env::current_dir()?)),
    }
}
pub fn init_command(matches: &ArgMatches) -> Result<()> {
    let project = resolve_project_directory(matches)?;
    if !project.spm_toml_exists() {
        project.write_spm_toml_contents("\n[extensions]")?;
    }
    if !project.sqlite_extensions_exists() {
        project.create_sqlite_extensions_dir()?;
        project.write_in_sqlite_extensions(".gitignore".into(), "*")?
    }
    Ok(())
}

pub fn add_command(matches: &ArgMatches) -> Result<()> {
    let url = matches
        .get_one::<String>("url")
        .context("url is a required argument")?;
    let artifact = matches.get_one::<String>("artifact");
    let artifact = artifact.expect("TODO optional version and artifacts");

    let project = resolve_project_directory(matches)?;

    let spm_toml_contents = project.read_spm_toml_contents()?;
    let mut doc = spm_toml_contents
        .parse::<Document>()
        .context("invalid spm.toml")?;
    doc["extensions"][url] = value(artifact);
    project.write_spm_toml_contents(doc.to_string())?;
    Ok(())
}

pub fn install_command(matches: &ArgMatches) -> Result<()> {
    let project = resolve_project_directory(matches)?;
    if !project.spm_toml_exists() {
        println!("No spm.toml found in current directory, exiting.");
        std::process::exit(1);
    }

    if !project.sqlite_extensions_exists() {
        project.create_sqlite_extensions_dir()?;
    }

    let spm_lock: SpmLock = project.read_spm_lock()?;

    let os = spm_os_name(std::env::consts::OS);
    let arch = std::env::consts::ARCH;

    for (name, extension) in spm_lock.extensions {
        let version = extension.version;
        for (_, e) in extension.spm_json.extensions {
            let platform = e
                .platforms
                .iter()
                .find(|platform| platform.os == os && platform.cpu == arch);
            let platform = platform.ok_or_else(|| {
                anyhow!("No matching platform found for the current device ({os}-{arch})")
            })?;

            let asset_name = &platform.asset_name;
            let url = format!("https://{name}/releases/download/{version}/{asset_name}");
            println!("downloading {url}");
            let asset = ureq::get(url.as_str())
                .call()
                .with_context(|| format!("Error making request to {url}"))?
                .into_reader();
            let buf_reader = BufReader::new(asset);
            let gz_decoder = GzDecoder::new(buf_reader);
            let mut archive = Archive::new(gz_decoder);

            // Extract the file
            let entry = archive
                .entries()
                .with_context(|| format!("Error finding entries in {asset_name}"))?
                .filter_map(|entry| entry.ok())
                .next();
            entry
                .with_context(|| format!("could not unpack tar.gz entry for {}", asset_name))?
                .unpack_in(&project.sqlite_extensions_path())
                .with_context(|| {
                    format!(
                        "could not unpack tar.gz entry into {}",
                        project.sqlite_extensions_path().display()
                    )
                })?;
        }
    }
    Ok(())
}

pub fn generate_lockfile(spm_toml: &SpmToml) -> Result<SpmLock> {
    let mut extensions = HashMap::new();
    for (extension_name, version) in &spm_toml.extensions {
        let resolved_url = format!("https://{extension_name}");
        let resolved_spm_json =
            format!("https://{extension_name}/releases/download/{version}/spm.json");

        let integrity = "".to_owned();

        let url = resolved_spm_json.as_str();
        let spm_json: SpmPackageJson = ureq::get(url)
            .call()
            .with_context(|| format!("Could not fetch spm.json file at {url}"))?
            .into_json()
            .with_context(|| format!("Could not decode fetched spm.json into JSON, from {url}"))?;
        extensions.insert(
            extension_name.clone(),
            SpmLockExtension {
                version: version.clone(),
                resolved_url,
                resolved_spm_json,
                integrity,
                spm_json,
            },
        );
    }
    Ok(SpmLock { extensions })
}

pub fn generate_command(matches: &ArgMatches) -> Result<()> {
    let project = resolve_project_directory(matches)?;
    let spm_toml = project.read_spm_toml()?;

    let lockfile = generate_lockfile(&spm_toml)?;
    project.write_spm_lock(lockfile)?;
    Ok(())
}

fn spm_os_name(os: &str) -> &str {
    if os == "macos" {
        "darwin"
    } else {
        os
    }
}
