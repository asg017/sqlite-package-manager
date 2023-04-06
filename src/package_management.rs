use std::{collections::HashMap, io::Write};

use crate::spm::{SpmLock, SpmLockExtension, SpmPackageJson, SpmToml};

use clap::ArgMatches;

use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::io::BufReader;
use tar::Archive;

use toml_edit::{value, Document};

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    description: String,
    extensions: ConfigExtensions,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ConfigExtensions {
    #[serde(flatten)]
    extensions: HashMap<String, ConfigExtension>,
}

#[derive(Serialize, Deserialize)]
struct ConfigExtension {
    version: String,
    url: String,
    artifact: String,
}
pub fn init_command() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let spm_toml = std::fs::metadata(cwd.join("spm.toml"));
    let sqlite_ext_dir = std::fs::metadata(cwd.join("sqlite_extensions"));
    if spm_toml.is_err() {
        let ce = ConfigExtensions {
            extensions: HashMap::new(),
        };
        let config = Config {
            description: "boingo".to_owned(),
            extensions: ce,
        };
        let mut f = std::fs::File::create(cwd.join("spm.toml")).unwrap();
        let contents = toml::to_string(&config).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
    }
    if sqlite_ext_dir.is_err() {
        std::fs::create_dir(cwd.join("sqlite_extensions")).unwrap();
        let mut f =
            std::fs::File::create(cwd.join("sqlite_extensions").join(".gitignore")).unwrap();
        f.write_all(b"*").unwrap();
    }
    Ok(())
}

pub fn add_command(matches: &ArgMatches) -> Result<()> {
    let url = matches.get_one::<String>("url").unwrap();
    let artifact = matches.get_one::<String>("artifact");
    let artifact = artifact.unwrap();

    let cwd = std::env::current_dir().unwrap();

    let spm_toml_contents = std::fs::read_to_string(cwd.join("spm.toml")).unwrap();
    let mut doc = spm_toml_contents.parse::<Document>().expect("invalid doc");
    doc["extensions"][url] = value(artifact);
    std::fs::write(cwd.join("spm.toml"), doc.to_string()).unwrap();
    Ok(())
}

pub fn install_command(_: &ArgMatches) -> Result<()> {
    let cwd = std::env::current_dir().unwrap();
    if !std::path::Path::exists(&cwd.join("spm.toml")) {
        println!("No spm.toml found in current directory, exiting.");
        std::process::exit(1);
    }
    let sqlite_extensions_directory = cwd.join("sqlite_extensions");

    if !sqlite_extensions_directory.exists() {
        std::fs::create_dir(&sqlite_extensions_directory).ok();
    }

    //let spm_toml_contents = ;
    let spm_lock: SpmLock =
        serde_json::from_str(&std::fs::read_to_string(cwd.join("spm.lock")).unwrap()).unwrap();

    let os = spm_os_name(std::env::consts::OS);
    let arch = std::env::consts::ARCH;

    for (name, extension) in spm_lock.extensions {
        let version = extension.version;
        for (_, e) in extension.spm_json.extensions {
            let platform = e
                .platforms
                .iter()
                .find(|platform| platform.os == os && platform.cpu == arch);
            let platform = platform.ok_or_else(|| anyhow!("asdf"))?;

            let asset_name = &platform.asset_name;
            let url = format!("https://{name}/releases/download/{version}/{asset_name}");
            println!("downloading {url}");
            let asset = ureq::get(url.as_str()).call().unwrap().into_reader();
            let buf_reader = BufReader::new(asset);
            let gz_decoder = GzDecoder::new(buf_reader);
            let mut archive = Archive::new(gz_decoder);

            // Extract the file
            let entry = archive
                .entries()
                .unwrap()
                .filter_map(|entry| entry.ok())
                .next(); //.next();
                         //.find(|entry| entry.path().unwrap().to_str().unwrap() == "ulid0.dylib");
            entry
                .unwrap()
                .unpack_in(&sqlite_extensions_directory)
                .unwrap();
        }
    }
    Ok(())
}

pub fn generate_lockfile(spm_toml: SpmToml) -> SpmLock {
    let mut extensions = HashMap::new();
    for (extension_name, version) in spm_toml.extensions {
        let resolved_url = format!("https://{extension_name}");
        let resolved_spm_json =
            format!("https://{extension_name}/releases/download/{version}/spm.json");

        let integrity = "".to_owned();

        let spm_json: SpmPackageJson = ureq::get(resolved_spm_json.as_str())
            .call()
            .unwrap()
            .into_json()
            .unwrap();
        extensions.insert(
            extension_name,
            SpmLockExtension {
                version,
                resolved_url,
                resolved_spm_json,
                integrity,
                spm_json,
            },
        );
    }
    SpmLock { extensions }
}

pub fn generate_command() -> Result<()> {
    let cwd = std::env::current_dir().unwrap();

    let spm_toml_contents = std::fs::read_to_string(cwd.join("spm.toml")).unwrap();
    let spm_toml: SpmToml = toml::from_str(&spm_toml_contents).unwrap();

    let lockfile = generate_lockfile(spm_toml);
    std::fs::write(
        cwd.join("spm.lock"),
        serde_json::to_vec_pretty(&lockfile).unwrap(),
    )
    .unwrap();
    Ok(())
}

fn spm_os_name(os: &str) -> &str {
    if os == "macos" {
        "darwin"
    } else {
        os
    }
}
