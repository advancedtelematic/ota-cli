use reqwest::{header::ContentType, Client};
use serde_json;
use std::{fs::File, io::BufReader, path::Path};
use url::Url;
use url_serde;
use zip::ZipArchive;

use error::{Error, Result};

/// Closure that returns a new `AccessToken`.
pub type Token = Box<Fn() -> Result<Option<AccessToken>>>;

/// Access token from Auth+ used to authenticate HTTP requests.
#[derive(Serialize, Deserialize, Debug)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub scope: String,
}

impl AccessToken {
    pub fn refresh(client: &Client, credentials_zip: impl AsRef<Path>) -> Result<Option<AccessToken>> {
        let credentials = Credentials::parse(credentials_zip)?;
        match (credentials.oauth2, credentials.no_auth) {
            (Some(oauth), _) => {
                debug!("fetching access token from auth-plus: {}", oauth.server);
                Ok(Some(client
                    .post(&format!("{}/token", oauth.server))
                    .basic_auth(oauth.client_id, Some(oauth.client_secret))
                    .header(ContentType::form_url_encoded())
                    .body("grant_type=client_credentials")
                    .send()?
                    .json()?))
            }
            (None, Some(no_auth)) if no_auth == true => Ok(None),
            _ => Err(Error::Auth("no parseable auth method from credentials.zip".into())),
        }
    }
}

/// Parsed representation of `treehub.json` from `credentials.zip`.
#[derive(Serialize, Deserialize, Debug)]
struct Credentials {
    no_auth: Option<bool>,
    oauth2: Option<OAuth2>,
    ostree: Ostree,
}

impl Credentials {
    fn parse(credentials_zip: impl AsRef<Path>) -> Result<Self> {
        debug!("reading treehub.json from zip file: {:?}", credentials_zip.as_ref());
        let file = File::open(credentials_zip)?;
        let mut archive = ZipArchive::new(BufReader::new(file))?;
        let treehub = archive.by_name("treehub.json")?;
        Ok(serde_json::from_reader(treehub)?)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct OAuth2 {
    server: String,
    client_id: String,
    client_secret: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Ostree {
    #[serde(with = "url_serde")]
    server: Url,
}
