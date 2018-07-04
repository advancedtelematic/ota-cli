use clap::ArgMatches;
use reqwest::multipart::Form;
use url::Url;
use urlencoding;

use api::director::TargetFormat;
use config::Config;
use error::{Error, Result};
use util::print_resp;


/// Available TUF Reposerver API methods.
pub trait ReposerverApi {
    fn add_package(&mut Config, package: TargetPackage) -> Result<()>;
    fn get_package(&mut Config, name: &str, version: &str) -> Result<()>;
}

/// Make API calls to the TUF Reposerver.
pub struct Reposerver;

impl ReposerverApi for Reposerver {
    fn add_package(config: &mut Config, package: TargetPackage) -> Result<()> {
        let filepath = format!("{}-{}", package.name, package.version);
        debug!("adding package with filepath {}", filepath);
        config
            .client()
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
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn get_package(config: &mut Config, name: &str, version: &str) -> Result<()> {
        let filepath = format!("{}-{}", name, version);
        debug!("fetching package with filepath {}", filepath);
        config
            .client()
            .get(&format!("{}api/v1/user_repo/targets/{}", config.reposerver, filepath))
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }
}


/// Encapsulate package details for uploading to the TUF Reposerver.
pub struct TargetPackage {
    name:     String,
    version:  String,
    hardware: Vec<String>,
    target:   RepoTarget,
    format:   TargetFormat,
}

impl<'a> TargetPackage {
    /// Parse CLI arguments into a new `Package`.
    pub fn from_flags(flags: &ArgMatches<'a>) -> Result<Self> {
        Ok(TargetPackage {
            name:     flags.value_of("name").expect("--name").into(),
            version:  flags.value_of("version").expect("--version").into(),
            hardware: flags.values_of("hardware").expect("--hardware").map(String::from).collect(),
            target:   RepoTarget::from_flags(&flags)?,
            format:   TargetFormat::from_flags(&flags)?,
        })
    }
}

/// Target data pointed to by either filesystem path or remote URL.
pub enum RepoTarget {
    Path(String),
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
