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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpmToml {
    pub description: String,
    pub extensions: HashMap<String, String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpmLock {
    pub extensions: HashMap<String, SpmLockExtension>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpmLockExtension {
    pub version: String,
    #[serde(rename = "resolved_url")]
    pub resolved_url: String,
    #[serde(rename = "resolved_spm_json")]
    pub resolved_spm_json: String,
    pub integrity: String,
    #[serde(rename = "spm_json")]
    pub spm_json: SpmPackageJson,
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
        assert_eq!(ext1.version, "vX.X.X");
    }

    #[test]
    fn test_spm_toml() {
        let data = r#"
        description = "boingo"

        [extensions]

        "github.com/asg017/sqlite-path" = "v0.2.0-alpha.1"
        "github.com/asg017/sqlite-url" =  "v0.1.0-alpha.3"
        "github.com/asg017/sqlite-html" = "v0.1.2-alpha.4"
        "github.com/asg017/sqlite-http" = "v0.1.0-alpha.2"
        "#;

        let t: SpmToml = toml::from_str(data).unwrap();

        assert_eq!(t.description, "boingo");
        assert_eq!(
            t.extensions.get("github.com/asg017/sqlite-path").unwrap(),
            "v0.2.0-alpha.1"
        );
    }
}
