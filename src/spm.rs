use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpmPackageJson {
    pub version: i64,
    pub extensions: HashMap<String, SpmPackageJsonExtension>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExtensionDefinition {
    Version(String),
    Definition {
        version: String,
        artifacts: Vec<String>,
    },
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpmToml {
    pub description: String,
    pub extensions: HashMap<String, ExtensionDefinition>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

use std::path::PathBuf;

use anyhow::{Context, Result};

pub(crate) struct ProjectDirectory {
    spm_toml_path: PathBuf,
    spm_lock_path: PathBuf,
    sqlite_extensions_path: PathBuf,
}
impl ProjectDirectory {
    pub fn new(base_directory: PathBuf) -> ProjectDirectory {
        let spm_toml_path = base_directory.join("spm.toml");
        let spm_lock_path = base_directory.join("spm.lock");
        let sqlite_extensions_path = base_directory.join("sqlite_extensions");
        ProjectDirectory {
            spm_toml_path,
            spm_lock_path,
            sqlite_extensions_path,
        }
    }
    pub fn sqlite_extensions_path(&self) -> std::path::PathBuf {
        self.sqlite_extensions_path.clone()
    }
    pub fn spm_toml_exists(&self) -> bool {
        std::path::Path::exists(&self.spm_toml_path)
    }
    pub fn _spm_lock_exists(&self) -> bool {
        std::path::Path::exists(&self.spm_lock_path)
    }
    pub fn sqlite_extensions_exists(&self) -> bool {
        std::path::Path::exists(&self.sqlite_extensions_path)
    }
    pub fn create_sqlite_extensions_dir(&self) -> Result<()> {
        std::fs::create_dir(&self.sqlite_extensions_path).with_context(|| {
            format!(
                "Could not create new directory at {}",
                self.sqlite_extensions_path.display()
            )
        })?;
        Ok(())
    }
    pub fn read_spm_toml(&self) -> Result<SpmToml> {
        let contents = self.read_spm_toml_contents()?;
        let spm_toml = toml::from_str(&contents)
            .with_context(|| format!("spm.toml at {} is not valid", "TODO"))?;
        Ok(spm_toml)
    }
    pub fn read_spm_lock(&self) -> Result<SpmLock> {
        let contents = self.read_spm_lock_contents()?;
        let spm_toml = serde_json::from_str(&contents)
            .with_context(|| format!("spm.lock at {} is not valid", "TODO"))?;
        Ok(spm_toml)
    }
    pub fn read_spm_toml_contents(&self) -> Result<String> {
        Ok(std::fs::read_to_string(&self.spm_toml_path)?)
    }
    pub fn read_spm_lock_contents(&self) -> Result<String> {
        Ok(std::fs::read_to_string(&self.spm_lock_path)?)
    }
    pub fn write_spm_toml_contents<C: AsRef<[u8]>>(&self, contents: C) -> Result<()> {
        std::fs::write(&self.spm_toml_path, contents)
            .with_context(|| format!("could not write to {}", &self.spm_toml_path.display()))
    }
    pub fn write_spm_lock(&self, lock: SpmLock) -> Result<()> {
        let contents = serde_json::to_vec_pretty(&lock).context("Failed to serialize spm.lock")?;
        std::fs::write(&self.spm_lock_path, contents)
            .with_context(|| format!("could not write to {}", &self.spm_lock_path.display()))
    }
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

        if let ExtensionDefinition::Version(v) = x {
            assert_eq!(v, "v0.2.0-alpha.1");
        } else {
            panic!();
        }
    }
}
