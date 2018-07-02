use clap::ArgMatches;
use reqwest::{
    header::{Authorization, Bearer, Headers},
    Client,
};
use serde_json;
use std::{
    env,
    fs::{File, OpenOptions},
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};
use url::Url;
use url_serde;
use zip::ZipArchive;

use api::auth_plus::{AccessToken, Credentials};
use error::Result;


const CONFIG_FILE: &str = ".ota.conf";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub credentials_zip: PathBuf,
    #[serde(skip)]
    pub credentials: Option<Credentials>,
    #[serde(skip)]
    pub client: Option<Client>,
    pub token: Option<AccessToken>,

    #[serde(with = "url_serde")]
    pub campaigner: Url,
    #[serde(with = "url_serde")]
    pub director: Url,
    #[serde(with = "url_serde")]
    pub registry: Url,
    #[serde(with = "url_serde")]
    pub reposerver: Url,
}

impl<'a> Config {
    /// Initialize a new config from CLI arguments.
    pub fn init_matches(matches: &ArgMatches<'a>) -> Result<()> {
        let credentials: PathBuf = matches.value_of("credentials").expect("--credentials").into();
        let campaigner = matches.value_of("campaigner").expect("--campaigner").parse()?;
        let director = matches.value_of("director").expect("--director").parse()?;
        let registry = matches.value_of("registry").expect("--registry").parse()?;
        Self::init(credentials, campaigner, director, registry)
    }

    /// Initialize a new config file.
    pub fn init(credentials_zip: impl Into<PathBuf>, campaigner: Url, director: Url, registry: Url) -> Result<()> {
        let zip = credentials_zip.into();
        let reposerver = reposerver_url(&zip)?;
        let config = Config {
            credentials_zip: zip,
            credentials: None,
            client: None,
            token: None,
            campaigner,
            director,
            registry,
            reposerver,
        };
        config.save_default()
    }

    /// Save the current config.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(path)?;
        Ok(file.write_all(&serde_json::to_vec_pretty(&self)?)?)
    }

    /// Load a previously saved config.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(serde_json::from_slice(&buf)?)
    }

    /// Save the default config file.
    pub fn save_default(&self) -> Result<()> { self.save(default_path()) }

    /// Load the default config file.
    pub fn load_default() -> Result<Self> { Self::load(default_path()) }

    /// Returns a clone of the cached HTTP client.
    pub fn client(&mut self) -> Client {
        if let None = self.client {
            self.client = Some(Client::new());
        }
        self.client.clone().unwrap()
    }

    /// Returns a clone of the cached access token.
    pub fn token(&mut self) -> Result<Option<AccessToken>> {
        if let None = self.token {
            self.token = AccessToken::refresh(&self.client(), self.credentials()?)?;
            self.save_default()?;
        }
        Ok(self.token.clone())
    }

    /// Returns a reference to the cached credentials.
    pub fn credentials(&mut self) -> Result<&Credentials> {
        if let None = self.credentials {
            self.credentials = Some(Credentials::parse(&self.credentials_zip)?);
        }
        Ok(self.credentials.as_ref().unwrap())
    }

    /// Returns a list of headers with a Bearer token added.
    pub fn bearer_token(&mut self) -> Result<Headers> {
        let mut headers = Headers::new();
        if let Some(t) = self.token()? {
            headers.set(Authorization(Bearer {
                token: t.access_token.clone(),
            }));
        }
        Ok(headers)
    }
}


fn default_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env::home_dir().expect("couldn't read home directory path"));
    path.push(CONFIG_FILE);
    path
}

fn reposerver_url(credentials_zip: impl AsRef<Path>) -> Result<Url> {
    debug!("reading tufrepo.url from credentials.zip");
    let file = File::open(credentials_zip)?;
    let mut archive = ZipArchive::new(BufReader::new(file))?;
    let mut tufrepo = archive.by_name("tufrepo.url")?;
    let mut contents = String::new();
    let _ = tufrepo.read_to_string(&mut contents)?;
    Ok(Url::from_str(&contents)?)
}
