use reqwest;
use serde_json;
use std::{
    self,
    fmt::{self, Debug, Display, Formatter},
};
use toml;
use url;
use uuid;
use zip;


/// Project `Result` type with a fixed error branch.
pub type Result<T> = std::result::Result<T, Error>;

/// All possible project `Error` types.
pub enum Error {
    Auth(String),
    Command(String),
    Flag(String),
    Http(reqwest::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    Parse(String),
    Toml(toml::de::Error),
    Url(url::ParseError),
    Uuid(uuid::ParseError),
    Zip(zip::result::ZipError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let output = match self {
            Error::Auth(ref err) => format!("Auth error: {}", err),
            Error::Command(ref err) => format!("Command input: {}", err),
            Error::Flag(ref err) => format!("Command flags: {}", err),
            Error::Http(ref err) => format!("HTTP: {}", err),
            Error::Io(ref err) => format!("I/O: {}", err),
            Error::Json(ref err) => format!("JSON parsing: {}", err),
            Error::Parse(ref err) => format!("Parse error: {}", err),
            Error::Toml(ref err) => format!("TOML parsing: {}", err),
            Error::Url(ref err) => format!("URL parsing: {}", err),
            Error::Uuid(ref err) => format!("UUID parsing: {}", err),
            Error::Zip(ref err) => format!("Zip/Unzip: {}", err),
        };
        write!(f, "{}", output)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { write!(f, "{}", self) }
}

impl std::error::Error for Error {
    fn description(&self) -> &str { "ota-cli error" }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self { Error::Http(err) }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self { Error::Io(err) }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self { Error::Json(err) }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self { Error::Toml(err) }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self { Error::Url(err) }
}

impl From<uuid::ParseError> for Error {
    fn from(err: uuid::ParseError) -> Self { Error::Uuid(err) }
}

impl From<zip::result::ZipError> for Error {
    fn from(err: zip::result::ZipError) -> Self { Error::Zip(err) }
}
