use reqwest::{header::ContentType, Client};
use serde_json;
use std::{fs::File, io::BufReader, path::Path};
use url::Url;
use url_serde;
use zip::ZipArchive;

use config::Config;
use error::{Error, Result};


/// Available Auth+ API methods.
pub trait AuthPlusApi {
    fn refresh_token(&mut Config) -> Result<Option<AccessToken>>;
}

/// Make API calls to Auth+.
pub struct AuthPlus;

impl AuthPlusApi for AuthPlus {
    fn refresh_token(config: &mut Config) -> Result<Option<AccessToken>> {
        if let Some(oauth2) = config.credentials()?.oauth2()? {
            debug!("fetching access token from auth-plus: {}", oauth2.server);
            let token = Client::new()
                .post(&format!("{}/token", oauth2.server))
                .basic_auth(oauth2.client_id, Some(oauth2.client_secret))
                .header(ContentType::form_url_encoded())
                .body("grant_type=client_credentials")
                .send()?
                .json()?;
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }
}


/// Access token used to authenticate HTTP requests.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type:   String,
    pub expires_in:   i32,
    pub scope:        String,
}

impl AccessToken {
    pub fn namespace(&self) -> Result<&str> {
        let scopes = self
            .scope
            .split_whitespace()
            .filter(|s| s.starts_with("namespace."))
            .map(|s| s.trim_left_matches("namespace."))
            .collect::<Vec<_>>();

        match scopes.len() {
            1 => Ok(scopes.first().unwrap()),
            0 => Err(Error::Token("namespace not found".into())),
            _ => Err(Error::Token(format!("multiple namespaces found: {:?}", scopes))),
        }
    }
}


/// Parsed credentials from `treehub.json` in `credentials.zip`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Credentials {
    no_auth: Option<bool>,
    oauth2:  Option<OAuth2>,
    ostree:  Ostree,
}

impl Credentials {
    pub fn parse(credentials_zip: impl AsRef<Path>) -> Result<Self> {
        debug!("reading treehub.json from zip file: {:?}", credentials_zip.as_ref());
        let file = File::open(credentials_zip)?;
        let mut archive = ZipArchive::new(BufReader::new(file))?;
        let treehub = archive.by_name("treehub.json")?;
        Ok(serde_json::from_reader(treehub)?)
    }

    fn oauth2(&self) -> Result<Option<OAuth2>> {
        if let Some(true) = self.no_auth {
            Ok(None)
        } else if let Some(ref oauth2) = self.oauth2 {
            Ok(Some(oauth2.clone()))
        } else {
            Err(Error::Auth("no parseable auth method from credentials.zip".into()))
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct OAuth2 {
    server:        String,
    client_id:     String,
    client_secret: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Ostree {
    #[serde(with = "url_serde")]
    server: Url,
}
