use error::Error;
use reqwest::{header::ContentType, Client};
use serde_json;
use std::{fs::File, io::BufReader, path::Path};
use url::Url;
use url_serde;
use zip::ZipArchive;

/// Closure that returns a new `AccessToken`.
pub type Token = Box<Fn() -> Result<AccessToken, Error>>;

/// Access token from Auth+ used to authenticate HTTP requests.
#[derive(Serialize, Deserialize, Debug)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub scope: String,
}

impl AccessToken {
    pub fn refresh(client: &Client, credentials_zip: impl AsRef<Path>) -> Result<AccessToken, Error> {
        let credentials = Credentials::parse(credentials_zip)?;
        debug!("fetching access token from Auth+ server: {}", credentials.oauth2.server);
        Ok(client
            .post(&format!("{}/token", credentials.oauth2.server))
            .header(ContentType::form_url_encoded())
            .basic_auth(credentials.oauth2.client_id, Some(credentials.oauth2.client_secret))
            .body("grant_type=client_credentials")
            .send()?
            .json()?)
    }
}

/// Parsed representation of `treehub.json` from `credentials.zip`.
#[derive(Serialize, Deserialize, Debug)]
struct Credentials {
    oauth2: OAuth2,
    ostree: Ostree,
}

impl Credentials {
    fn parse(credentials_zip: impl AsRef<Path>) -> Result<Self, Error> {
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
