use clap::ArgMatches;
use reqwest::{multipart::Form, Client, Response};
use std::{collections::HashMap, fs, path::Path};
use toml;
use url::Url;
use url_serde;
use urlencoding;

use api::director::TargetFormat;
use config::Config;
use error::{Error, Result};
use http::{Http, HttpMethods};


/// Available TUF Reposerver API methods.
pub trait ReposerverApi {
    fn add_package(&mut Config, package: TufPackage) -> Result<Response>;
    fn get_package(&mut Config, name: &str, version: &str) -> Result<Response>;
}

/// Make API calls to the TUF Reposerver.
pub struct Reposerver;

impl ReposerverApi for Reposerver {
    fn add_package(config: &mut Config, package: TufPackage) -> Result<Response> {
        let filepath = format!("{}-{}", package.name, package.version);
        debug!("adding package with filepath {}", filepath);
        let req = Client::new()
            .put(&format!("{}api/v1/user_repo/targets/{}", config.reposerver, filepath))
            .query(&[
                ("name", urlencoding::encode(&package.name)),
                ("version", urlencoding::encode(&package.version)),
                ("hardwareIds", package.hardware.join(",")),
                ("targetFormat", format!("{}", package.format)),
            ])
            .multipart(match package.target {
                RepoTarget::Path(path) => Form::new().file("file", path)?,
                RepoTarget::Url(url) => Form::new().file("fileUri", url.as_str())?,
            })
            .build()?;
        Http::send(req, config.token()?)
    }

    fn get_package(config: &mut Config, name: &str, version: &str) -> Result<Response> {
        let filepath = format!("{}-{}", name, version);
        debug!("fetching package with filepath {}", filepath);
        Http::get(&format!("{}api/v1/user_repo/targets/{}", config.reposerver, filepath), config.token()?)
    }
}


/// A parsed TOML representation of a package.
#[derive(Serialize, Deserialize)]
pub struct TargetPackage {
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    version: String,
    format: TargetFormat,
    hardware: Vec<String>,
}

/// A parsed mapping from package names to package metadata.
#[derive(Serialize, Deserialize)]
pub struct TargetPackages {
    pub packages: HashMap<String, TargetPackage>,
}

impl TargetPackages {
    /// Parse a toml file into `TargetPackages`.
    pub fn from_file(input: impl AsRef<Path>) -> Result<Self> {
        let packages = toml::from_str(&fs::read_to_string(input)?)?;
        Ok(Self { packages })
    }
}


/// A package target for uploading to the TUF Reposerver.
#[derive(Serialize, Deserialize)]
pub struct TufPackage {
    name:     String,
    version:  String,
    hardware: Vec<String>,
    target:   RepoTarget,
    format:   TargetFormat,
}

impl<'a> TufPackage {
    /// Parse CLI arguments into a `TufPackage`.
    pub fn from_flags(flags: &ArgMatches<'a>) -> Result<Self> {
        Ok(TufPackage {
            name:     flags.value_of("name").expect("--name").into(),
            version:  flags.value_of("version").expect("--version").into(),
            hardware: flags.values_of("hardware").expect("--hardware").map(String::from).collect(),
            target:   RepoTarget::from_flags(&flags)?,
            format:   TargetFormat::from_flags(&flags)?,
        })
    }
}

/// A collection of TUF packages for uploading.
#[derive(Serialize, Deserialize)]
pub struct TufPackages {
    pub packages: Vec<TufPackage>,
}

impl TufPackages {
    /// Convert `TargetPackages` to `TufPackages`.
    pub fn from(targets: TargetPackages) -> Result<Self> {
        Ok(Self {
            packages: targets
                .packages
                .into_iter()
                .map(|(name, pack)| Self::to_tuf_package(name, pack))
                .collect::<Result<Vec<_>>>()?,
        })
    }

    fn to_tuf_package(name: String, package: TargetPackage) -> Result<TufPackage> {
        #[cfg_attr(rustfmt, rustfmt_skip)] 
        Ok(TufPackage {
            name,
            version:  package.version,
            hardware: package.hardware,
            target: match (package.path, package.url) {
                (Some(path), None) => RepoTarget::Path(path),
                (None, Some(url))  => RepoTarget::Url(url.parse()?),
                (None, None)       => Err(Error::Parse("One of `path` or `url` required.".into()))?,
                (Some(_), Some(_)) => Err(Error::Parse("Either `path` or `url` expected. Not both.".into()))?,
            },
            format: package.format,
        })
    }
}

/// Target data pointed to by either filesystem path or remote URL.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum RepoTarget {
    Path(String),
    #[serde(with = "url_serde")]
    Url(Url),
}

impl<'a> RepoTarget {
    pub fn from_flags(flags: &ArgMatches<'a>) -> Result<Self> {
        if let Some(path) = flags.value_of("path") {
            Ok(RepoTarget::Path(path.into()))
        } else if let Some(url) = flags.value_of("url") {
            Ok(RepoTarget::Url(url.parse()?))
        } else {
            Err(Error::Flag("Either --path or --url flag is required".into()))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn parse_example_packages() {
        let targets = TargetPackages::from_file("examples/packages.toml").expect("parse toml");
        let mut packages = TufPackages::from(targets).expect("convert package").packages;
        assert_eq!(packages.len(), 2);
        packages.sort_unstable_by(|a,b| a.name.cmp(&b.name));

        assert_eq!(packages[0].name, "foo".to_string());
        assert_eq!(packages[0].version, "1".to_string());
        assert_eq!(packages[0].hardware, vec!["acme-ecu-1".to_string()]);
        assert_eq!(packages[0].target, RepoTarget::Url("https://acme.org/downloads/foo".parse().expect("parse url")));
        assert_eq!(packages[0].format, TargetFormat::Binary);

        assert_eq!(packages[1].name, "my-branch".to_string());
        assert_eq!(packages[1].version, "01234".to_string());
        assert_eq!(packages[1].hardware, vec!["qemux86-64".to_string()]);
        assert_eq!(packages[1].target, RepoTarget::Path("/ota/my-branch-01234".into()));
        assert_eq!(packages[1].format, TargetFormat::Ostree);
    }
}
