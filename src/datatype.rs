use failure::Error;
use reqwest::Client;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use serde_json;
use std::{fmt::{self, Display, Formatter},
          fs::File,
          io::BufReader,
          ops::Deref,
          path::PathBuf,
          str::FromStr};
use url;
use zip::ZipArchive;

/// Access token from Auth+ used to authenticate HTTP requests.
#[derive(Serialize, Deserialize, Debug)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub scope: String,
}

impl AccessToken {
    pub fn refresh(client: &Client, credentials: PathBuf) -> Result<AccessToken, Error> {
        let credentials = Credentials::parse(credentials)?;
        debug!("fetching access token from Auth+ server: {}", credentials.oauth2.server);
        Ok(client
            .post(&format!("{}/token", credentials.oauth2.server))
            .body("grant_type=client_credentials")
            .send()?
            .json()?)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Credentials {
    oauth2: OAuth2,
    ostree: Ostree,
}

impl Credentials {
    pub fn parse(zip: PathBuf) -> Result<Self, Error> {
        debug!("reading treehub.json from zip file: {:?}", zip);
        let file = File::open(zip)?;
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
    server: Url,
}

/// Wrapper type that implements serde `Serialize` and `Deserialize`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Url(pub url::Url);

impl FromStr for Url {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Url(url::Url::parse(s)?))
    }
}

impl Serialize for Url {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&format!("{}", self))
    }
}

impl<'de> Deserialize<'de> for Url {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Url, D::Error> {
        let s: String = Deserialize::deserialize(de)?;
        s.parse().map_err(|err| serde::de::Error::custom(format!("invalid url: {}", err)))
    }
}

impl Deref for Url {
    type Target = url::Url;

    fn deref(&self) -> &url::Url {
        &self.0
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let host = self.0.host_str().unwrap_or("localhost");
        if let Some(port) = self.0.port() {
            write!(f, "{}://{}:{}{}", self.0.scheme(), host, port, self.0.path())
        } else {
            write!(f, "{}://{}{}", self.0.scheme(), host, self.0.path())
        }
    }
}
