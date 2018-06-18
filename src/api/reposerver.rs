use reqwest::header::{Authorization, Bearer, Headers};
use reqwest::multipart::Form;
use reqwest::Client;
use std::{fs::File,
          io::{BufReader, Read},
          path::Path,
          str::FromStr};
use url::Url;
use zip::ZipArchive;

use api::auth_plus::Token;
use api::director::TargetFormat;
use error::{Error, Result};
use urlencoding;
use util::print_resp;

/// Available TUF Reposerver API methods.
pub trait Reposerver {
    fn put_target(
        &self,
        name: String,
        version: String,
        hardware_ids: Vec<String>,
        target_format: TargetFormat,
        target: RepoTarget,
    ) -> Result<()>;
}

/// Target data pointed to by either filesystem path or remote URL.
pub enum RepoTarget {
    Path(String),
    Url(Url),
}

/// Handle API calls to TUF Reposerver.
pub struct ReposerverHandler<'c> {
    client: &'c Client,
    server: Url,
    token: Token,
}

impl<'c> ReposerverHandler<'c> {
    pub fn new(client: &'c Client, credentials_zip: impl AsRef<Path>, token: Token) -> Result<Self> {
        debug!("reading tufrepo.url from zip file: {:?}", credentials_zip.as_ref());
        let file = File::open(credentials_zip)?;
        let mut archive = ZipArchive::new(BufReader::new(file))?;
        let mut tufrepo = archive.by_name("tufrepo.url")?;
        let mut contents = String::new();
        let _ = tufrepo.read_to_string(&mut contents)?;
        let server = Url::from_str(&contents)?;
        Ok(ReposerverHandler { client, server, token })
    }

    fn auth(&self) -> Result<Headers> {
        let mut headers = Headers::new();
        if let Some(resp) = (self.token)()? {
            headers.set(Authorization(Bearer {
                token: resp.access_token,
            }));
        }
        Ok(headers)
    }
}

impl<'c> Reposerver for ReposerverHandler<'c> {
    fn put_target(
        &self,
        name: String,
        version: String,
        hardware_ids: Vec<String>,
        target_format: TargetFormat,
        target: RepoTarget,
    ) -> Result<()> {
        let package = format!("{}-{}", name, version);
        debug!("adding package: {}", package);
        self.client
            .put(&format!("{}api/v1/user_repo/targets/{}", self.server, package))
            .query(&[
                ("name", urlencoding::encode(&name)),
                ("version", urlencoding::encode(&version)),
                ("hardwareIds", urlencoding::encode(&hardware_ids.join(","))),
                ("targetFormat", format!("{}", target_format)),
            ])
            .multipart(match target {
                RepoTarget::Path(path) => Form::new().file("file", path)?,
                RepoTarget::Url(url) => Form::new().file("fileUri", format!("{}", url))?,
            })
            .headers(self.auth()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }
}
