use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{collections::HashMap, ffi::OsString, process::Stdio, str::Split};

use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use std::io::BufReader;
use tar::Archive;
use toml_edit::{value, Array, Document, InlineTable, Item};
use url::Url;

pub(crate) struct Project {
    base_project_directory: PathBuf,
    spm_toml_path: PathBuf,
    spm_lock_path: PathBuf,
    sqlite_extensions_path: PathBuf,
}

#[cfg(target_os = "linux")]
const LIBRARY_PATH_ENV_VAR: &str = "LD_LIBRARY_PATH";
#[cfg(target_os = "macos")]
const LIBRARY_PATH_ENV_VAR: &str = "DYLD_LIBRARY_PATH";
#[cfg(target_os = "windows")]
const LIBRARY_PATH_ENV_VAR: &str = "PATH";

impl Project {
    pub fn new(base_project_directory: PathBuf) -> Project {
        let spm_toml_path = base_project_directory.join("spm.toml");
        let spm_lock_path = base_project_directory.join("spm.lock");
        let sqlite_extensions_path = base_project_directory.join("sqlite_extensions");
        Project {
            base_project_directory,
            spm_toml_path,
            spm_lock_path,
            sqlite_extensions_path,
        }
    }
    pub fn resolve_from_args(matches: &ArgMatches) -> Result<Project> {
        match matches.get_one::<String>("prefix") {
            Some(base_directory) => Ok(Project::new(base_directory.into())),
            // TODO traverse up the folder tree to find nearest directory with a spm.toml
            None => Ok(Project::new(std::env::current_dir()?)),
        }
    }
    /// Implements `spm init`
    pub fn command_init(&self) -> Result<()> {
        if !self.spm_toml_exists() {
            self.write_spm_toml_contents("\n[extensions]")?;
        }
        if !self.sqlite_extensions_exists() {
            self.create_sqlite_extensions_dir()?;
            self.write_in_sqlite_extensions(".gitignore".into(), "*")?
        }
        Ok(())
    }

    /// Implements `spm add`
    pub fn command_add(&self, url: &str, artifacts: Option<Vec<String>>) -> Result<()> {
        let pkg_resolver = determine_package_resolver(url)?;
        let version = pkg_resolver.version_from_reference()?;

        let spm_toml_contents = self.read_spm_toml_contents()?;
        let mut doc = spm_toml_contents
            .parse::<Document>()
            .context("invalid spm.toml")?;
        doc["extensions"][pkg_resolver.toml_name().as_str()] = match artifacts {
            Some(artifacts) => {
                let mut t = InlineTable::new();
                t.insert("version", version.into());
                t.insert(
                    "artifacts",
                    toml_edit::Value::Array(Array::from_iter(artifacts)),
                );
                Item::Value(toml_edit::Value::InlineTable(t))
            }
            None => value(version),
        };

        self.write_spm_toml_contents(doc.to_string())?;

        self.generate_lockfile()?;
        self.install(None)?;
        Ok(())
    }

    /// Implements `spm install`
    pub fn command_install(&self) -> Result<()> {
        self.generate_lockfile()?;
        self.install(None)?;
        Ok(())
    }

    /// Implements `spm ci`
    // TODO verify that spm.toml and spm.lock are consistent, and exit if not
    pub fn command_clean_install(&self) -> Result<()> {
        self.install(None)?;
        Ok(())
    }

    /// Implements `spm activate`
    pub fn command_activate(&self) -> Result<()> {
        let library_path = self.resolve_library_path()?;
        println!(
            "export {}={}",
            LIBRARY_PATH_ENV_VAR,
            shell_escape::escape(library_path.to_string_lossy())
        );
        Ok(())
    }
    /// Implements `spm deactivate`
    pub fn command_deactivate(&self) -> Result<()> {
        // TODO
        println!("unset {}", LIBRARY_PATH_ENV_VAR);
        Ok(())
    }
    /// Implements `spm run`
    pub fn command_run(&self, program: &str, arguments: &[&String]) -> Result<()> {
        let mut cmd = std::process::Command::new(program)
            .args(arguments)
            .env(LIBRARY_PATH_ENV_VAR, self.resolve_library_path()?)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let status = cmd.wait()?;
        std::process::exit(status.code().map_or(1, |code| code));
    }

    /// returns a colon-seperated string of directory to "preload" libraries from.
    /// Meant to overwrite LIBRARY_PATH_ENV_VAR (LD_LIBRARY_PATH, DYLD_LIBRARY_PATH, etc.)
    fn resolve_library_path(&self) -> Result<OsString> {
        let spm_toml = self.read_spm_toml()?;

        let mut preloads = match spm_toml.preload_directories {
            Some(preload_directories) => preload_directories
                .iter()
                .map(|path| {
                    let path = std::path::Path::new(path);
                    if path.is_absolute() {
                        Ok(path.to_path_buf())
                    } else {
                        let absolute = self.base_project_directory.clone().join(path);
                        Ok(std::fs::canonicalize(&absolute).with_context(|| {
                            format!(
                                "Could not find the preload directory: {}",
                                &absolute.display()
                            )
                        })?)
                    }
                })
                .collect::<Result<Vec<PathBuf>>>()?,
            None => vec![],
        };
        let preloads = match std::env::var_os(LIBRARY_PATH_ENV_VAR) {
            Some(paths) => {
                let mut paths = std::env::split_paths(&paths).collect::<Vec<_>>();
                paths.append(&mut preloads);
                paths.push(self.sqlite_extensions_path());
                paths
            }
            None => {
                preloads.push(self.sqlite_extensions_path());
                preloads
            }
        };
        std::env::join_paths(preloads)
            .context("Invalid path, is there a semicolor ':' somewhere in a path?")
    }

    // TODO skip extensions that are already downloaded
    fn install(&self, platform: Platform) -> Result<()> {
        if !self.spm_toml_exists() {
            println!("No spm.toml found in current directory, exiting.");
            std::process::exit(1);
        }

        if !self.sqlite_extensions_exists() {
            self.create_sqlite_extensions_dir()?;
        }

        let spm_lock: SpmLock = self.read_spm_lock()?;
        for extension in spm_lock.extensions.values() {
            match extension {
                SpmLockExtension::GithubRelease(extension) => {
                    extension.download_platform(platform.clone(), self)?;
                }
            }
        }
        Ok(())
    }

    // don't regenerate lockfile from scratch every time
    fn generate_lockfile(&self) -> Result<()> {
        let spm_toml = self.read_spm_toml()?;
        let mut extensions = HashMap::new();
        for (extension_name, definition) in &spm_toml.extensions {
            let pkg_resolver = determine_package_resolver(extension_name)?;
            let lock = pkg_resolver.generate_lock(definition)?;
            extensions.insert(extension_name.clone(), lock);
        }
        self.write_spm_lock(SpmLock {
            version: 0,
            extensions,
        })?;
        Ok(())
    }

    // full path of $BASE/sqlite_extensions/
    fn sqlite_extensions_path(&self) -> std::path::PathBuf {
        self.sqlite_extensions_path.clone()
    }

    /// does spm.toml for this project exist?
    fn spm_toml_exists(&self) -> bool {
        std::path::Path::exists(&self.spm_toml_path)
    }

    /// does spm.lock for this project exist?
    fn _spm_lock_exists(&self) -> bool {
        std::path::Path::exists(&self.spm_lock_path)
    }

    /// does sqlite_extensions/ for this project exist?
    fn sqlite_extensions_exists(&self) -> bool {
        std::path::Path::exists(&self.sqlite_extensions_path)
    }

    /// creates the sqlite_extensions/ directory
    fn create_sqlite_extensions_dir(&self) -> Result<()> {
        std::fs::create_dir(&self.sqlite_extensions_path).with_context(|| {
            format!(
                "Could not create new directory at {}",
                self.sqlite_extensions_path.display()
            )
        })?;
        Ok(())
    }

    /// read contents of the spm.toml file as SpmToml
    fn read_spm_toml(&self) -> Result<SpmToml> {
        let contents = self.read_spm_toml_contents()?;
        let spm_toml = toml::from_str(&contents)
            .with_context(|| format!("spm.toml at {} is not valid", "TODO"))?;
        Ok(spm_toml)
    }

    /// read contents of the spm.lock file as SpmLock
    fn read_spm_lock(&self) -> Result<SpmLock> {
        let contents = self.read_spm_lock_contents()?;
        let spm_toml = serde_json::from_str(&contents)
            .with_context(|| format!("spm.lock at {} is not valid", "TODO"))?;
        Ok(spm_toml)
    }

    /// read contents of the spm.toml file as a String
    fn read_spm_toml_contents(&self) -> Result<String> {
        Ok(std::fs::read_to_string(&self.spm_toml_path)?)
    }

    /// read contents of the spm.lock file as a String
    fn read_spm_lock_contents(&self) -> Result<String> {
        Ok(std::fs::read_to_string(&self.spm_lock_path)?)
    }

    /// write to the spm.toml with the provided contents
    pub fn write_spm_toml_contents<C: AsRef<[u8]>>(&self, contents: C) -> Result<()> {
        std::fs::write(&self.spm_toml_path, contents)
            .with_context(|| format!("could not write to {}", &self.spm_toml_path.display()))
    }

    /// write to the spm.lock with the provided contents
    pub fn write_spm_lock(&self, lock: SpmLock) -> Result<()> {
        let contents = serde_json::to_vec_pretty(&lock).context("Failed to serialize spm.lock")?;
        std::fs::write(&self.spm_lock_path, contents)
            .with_context(|| format!("could not write to {}", &self.spm_lock_path.display()))
    }

    /// write a single file into the sqlite_extensions/ directory with the given contents
    pub fn write_in_sqlite_extensions<C: AsRef<[u8]>>(
        &self,
        path: std::path::PathBuf,
        contents: C,
    ) -> Result<()> {
        let full_path = self.sqlite_extensions_path.join(path);
        std::fs::write(&full_path, contents)
            .with_context(|| format!("could not write to {}", full_path.display()))
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// spm.toml
pub struct SpmToml {
    pub description: String,
    pub preload_directories: Option<Vec<String>>,
    pub extensions: HashMap<String, SpmTomlExtensionDefinition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
/// definition of an extension in spm.toml, either a version string or object
pub enum SpmTomlExtensionDefinition {
    // project = "v1.2.3"
    Version(String),
    // project = { version="v1.2.3", artifacts=["abc0", "xyz0"] }
    Definition {
        version: String,
        artifacts: Vec<String>,
    },
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// spm.lock, serialized as JSON
pub struct SpmLock {
    pub version: i32,
    pub extensions: HashMap<String, SpmLockExtension>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpmLockExtension {
    GithubRelease(GithubReleaseExtension),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// spm.json
pub struct SpmPackageJson {
    pub version: i64,
    pub extensions: HashMap<String, SpmPackageJsonExtension>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// descibes a single extension in spm.json, under .extensions
pub struct SpmPackageJsonExtension {
    pub description: String,
    pub platforms: Vec<SpmPackageJsonPlatform>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpmPackageJsonPlatform {
    pub os: String,
    pub cpu: String,
    #[serde(rename = "asset_name")]
    pub asset_name: String,
    #[serde(rename = "asset_sha256")]
    pub asset_sha256: String,
    #[serde(rename = "asset_md5")]
    pub asset_md5: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubReleaseExtension {
    pub version: String,
    pub artifacts: Option<Vec<String>>,
    #[serde(rename = "resolved_url")]
    pub resolved_url: String,
    #[serde(rename = "resolved_spm_json")]
    pub resolved_spm_json: String,
    pub integrity: String,
    #[serde(rename = "spm_json")]
    pub spm_json: SpmPackageJson,
}
type Platform = Option<(String, String)>;
impl GithubReleaseExtension {
    pub(crate) fn download_platform(&self, platform: Platform, project: &Project) -> Result<()> {
        let (os, arch) = match platform {
            Some((os, arch)) => (os, arch),
            None => (
                spm_os_name(std::env::consts::OS).to_owned(),
                std::env::consts::ARCH.to_owned(),
            ),
        };
        for (name, e) in &self.spm_json.extensions {
            // if the extension definition only declares a subset of artifacts, then only
            // download those. ex `"xxx" = {artifacts=["a", "c"]}`, only download a and c, not b
            if let Some(artifacts) = &self.artifacts {
                if !artifacts.iter().any(|a| *a == *name) {
                    continue;
                }
            }
            let platform = e
                .platforms
                .iter()
                .find(|platform| platform.os == os && platform.cpu == arch);
            let platform = platform.ok_or_else(|| {
                anyhow!("No matching platform found for the current device ({os}-{arch})")
            })?;

            let asset_name = &platform.asset_name;
            let url = format!(
                "{}/releases/download/{}/{asset_name}",
                self.resolved_url, self.version
            );
            println!("downloading {url} ...");
            let asset = crate::http::http_get(url.as_str())
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
        Ok(())
    }
}

fn github_parse_path(mut parts: Split<char>) -> Result<GithubRelease> {
    let owner = parts
        .next()
        .ok_or_else(|| anyhow!("github owner name required"))?
        .to_owned();
    let repo = parts
        .next()
        .ok_or_else(|| anyhow!("github repo name required"))?
        .to_owned();
    if let Some((repo, version)) = repo.split_once('@') {
        Ok(GithubRelease {
            owner,
            repo: repo.to_owned(),
            version: Some(version.to_owned()),
        })
    } else {
        Ok(GithubRelease {
            owner,
            repo: repo.to_owned(),
            version: None,
        })
    }
}

// https://github.com/owner/repo -> GithubReleaseResolver
// github.com/owner/repo -> GithubReleaseResolver
// gh:owner/repo -> GithubReleaseResolver
fn determine_package_resolver(name: &str) -> Result<Box<dyn PackageResolver>> {
    if let Ok(url) = Url::parse(name) {
        return match url.host_str() {
            Some("github.com") => {
                let path_segments = url.path_segments().ok_or_else(|| anyhow!("wut"))?;
                Ok(Box::new(github_parse_path(path_segments)?))
            }
            Some(_) => Err(anyhow!("todo")),
            None => Err(anyhow!("todo")),
        };
    }
    if let Some(reference) = name.strip_prefix("gh:") {
        let parts = reference.split('/');
        return Ok(Box::new(github_parse_path(parts)?));
    }
    if let Some(reference) = name.strip_prefix("github.com/") {
        let parts = reference.split('/');
        return Ok(Box::new(github_parse_path(parts)?));
    }
    Err(anyhow!("todo"))
}
pub trait PackageResolver {
    fn version_from_reference(&self) -> Result<String>;
    fn toml_name(&self) -> String;
    fn latest_version(&self) -> Result<String>;
    fn generate_lock(&self, definition: &SpmTomlExtensionDefinition) -> Result<SpmLockExtension>;
}

struct GithubRelease {
    owner: String,
    repo: String,
    version: Option<String>,
}

impl PackageResolver for GithubRelease {
    fn version_from_reference(&self) -> Result<String> {
        match &self.version {
            Some(v) => Ok(v.to_owned()),
            None => self.latest_version(),
        }
    }
    fn toml_name(&self) -> String {
        format!("https://github.com/{}/{}", self.owner, self.repo)
    }
    fn latest_version(&self) -> Result<String> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.owner, self.repo
        );
        let response: serde_json::Value = crate::http::http_get(url.as_str())
            .call()
            .with_context(|| format!("call to {url} failed"))?
            .into_json()
            .with_context(|| format!("request did not return proper JSON: {url}"))?;

        Ok(response
            .get("tag_name")
            .context("Expected 'tag_name' in JSON response")?
            .as_str()
            .context("Expected 'tag_name' value to be a string")?
            .to_owned())
    }
    fn generate_lock(&self, definition: &SpmTomlExtensionDefinition) -> Result<SpmLockExtension> {
        let (version, artifacts) = match definition {
            SpmTomlExtensionDefinition::Version(version) => (version.clone(), None),
            SpmTomlExtensionDefinition::Definition { version, artifacts } => {
                (version.clone(), Some(artifacts.clone()))
            }
        };
        let resolved_url = format!("https://github.com/{}/{}", self.owner, self.repo);
        let resolved_spm_json = format!(
            "https://github.com/{}/{}/releases/download/{version}/spm.json",
            self.owner, self.repo
        );

        let integrity = "".to_owned();

        let url = resolved_spm_json.as_str();
        let spm_json: SpmPackageJson = crate::http::http_get(url)
            .call()
            .with_context(|| format!("Could not fetch spm.json file at {url}"))?
            .into_json()
            .with_context(|| format!("Could not decode fetched spm.json into JSON, from {url}"))?;

        Ok(SpmLockExtension::GithubRelease(GithubReleaseExtension {
            version,
            artifacts,
            resolved_url,
            resolved_spm_json,
            integrity,
            spm_json,
        }))
    }
}

fn spm_os_name(os: &str) -> &str {
    if os == "macos" {
        "darwin"
    } else {
        os
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spm_json() {
        let data = r#"
        {
          "version": 0,
          "extensions": {
            "path0": {
              "description": "",
              "platforms": [
                {
                  "os": "linux",
                  "cpu": "x86_64",
                  "asset_name": "sqlite-path-v0.2.0-alpha.1-linux-x86_64.tar.gz",
                  "asset_sha256": "040a5b89c2f70c176251414c6c517e0116958e4f810a8b579a86de00c74edbc2",
                  "asset_md5": "/tjynk9EhB/c6ATKjVU0wg=="
                },
                {
                  "os": "darwin",
                  "cpu": "x86_64",
                  "asset_name": "sqlite-path-v0.2.0-alpha.1-darwin-x86_64.tar.gz",
                  "asset_sha256": "6257095ac5eead76da801d5832463b2f6352c15bd84f25374b77c951276d2b0d",
                  "asset_md5": "7MPeVuRypdEWJJb2qo2v1Q=="
                },
                {
                  "os": "windows",
                  "cpu": "x86_64",
                  "asset_name": "sqlite-path-v0.2.0-alpha.1-windows-x86_64.tar.gz",
                  "asset_sha256": "ddefedeba9291fc62b6818d425d363142d392b0c568cd428f648c4acb87b65a6",
                  "asset_md5": "Tty30X/8OqRW4ElCheRQKg=="
                }
              ]
            }
          }
        }"#;

        let p: SpmPackageJson = serde_json::from_str(data).unwrap();

        assert_eq!(p.version, 0);
        assert_eq!(p.extensions.get("path0").unwrap().description, "");
    }

    #[test]
    fn test_spm_lock() {
        let data = r#"
        {
          "version": 0,
          "extensions": {
            "github.com/asg017/sqlite-path": {
              "version": "vX.X.X",
              "resolved_url": "https://github.com/asg017/sqlite-path",
              "resolved_spm_json": "https://github.com/asg017/sqlite-path/releases/download/v0.2.0-alpha.1/spm.json",
              "integrity": "L6kgHzUSLT5Ik02M8Wve7Q==",
              "spm_json": {
                "version": 0,
                "extensions": {
                  "path0": {
                    "description": "",
                    "platforms": [
                      {
                        "os": "linux",
                        "cpu": "x86_64",
                        "asset_name": "sqlite-path-v0.2.0-alpha.1-linux-x86_64.tar.gz",
                        "asset_sha256": "040a5b89c2f70c176251414c6c517e0116958e4f810a8b579a86de00c74edbc2",
                        "asset_md5": "/tjynk9EhB/c6ATKjVU0wg=="
                      },
                      {
                        "os": "darwin",
                        "cpu": "x86_64",
                        "asset_name": "sqlite-path-v0.2.0-alpha.1-darwin-x86_64.tar.gz",
                        "asset_sha256": "6257095ac5eead76da801d5832463b2f6352c15bd84f25374b77c951276d2b0d",
                        "asset_md5": "7MPeVuRypdEWJJb2qo2v1Q=="
                      },
                      {
                        "os": "windows",
                        "cpu": "x86_64",
                        "asset_name": "sqlite-path-v0.2.0-alpha.1-windows-x86_64.tar.gz",
                        "asset_sha256": "ddefedeba9291fc62b6818d425d363142d392b0c568cd428f648c4acb87b65a6",
                        "asset_md5": "Tty30X/8OqRW4ElCheRQKg=="
                      }
                    ]
                  }
                }
              }
            }
          }
        }
        "#;

        let p: SpmLock = serde_json::from_str(data).unwrap();
        let ext1 = p.extensions.get("github.com/asg017/sqlite-path").unwrap();
        match ext1 {
            SpmLockExtension::GithubRelease(gh) => {
                assert_eq!(gh.version, "vX.X.X");
            }
        };
    }

    #[test]
    fn test_spm_toml() {
        let data = r#"
        description = "boingo"

        [extensions]

        "github.com/asg017/sqlite-path" = "v0.2.0-alpha.1"
        "github.com/asg017/sqlite-url" =  {version = "v0.1.0-alpha.3", artifacts=["url0"]}
        "github.com/asg017/sqlite-html" = "v0.1.2-alpha.4"
        "github.com/asg017/sqlite-http" = "v0.1.0-alpha.2"
        "#;

        let t: SpmToml = toml::from_str(data).unwrap();

        assert_eq!(t.description, "boingo");
        let x = t.extensions.get("github.com/asg017/sqlite-path").unwrap();

        if let SpmTomlExtensionDefinition::Version(v) = x {
            assert_eq!(v, "v0.2.0-alpha.1");
        } else {
            panic!();
        }
    }
}
