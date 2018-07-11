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


/// Bind the error branch to `Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// Conversion from app or lib errors to a single representation.
pub enum Error {
    Auth(String),
    Command(String),
    Flag(String),
    NotFound(String, Option<String>),
    Parse(String),
    Token(String),

    Http(reqwest::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    Toml(toml::de::Error),
    Url(url::ParseError),
    Uuid(uuid::ParseError),
    Zip(zip::result::ZipError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let output = match self {
            Error::Auth(err) => format!("Authorization: {}", err),
            Error::Command(err) => format!("Command input: {}", err),
            Error::Flag(err) => format!("Command flags: {}", err),
            Error::NotFound(name, help) => if let Some(help) = help {
                format!("{} not found. {}", name, help)
            } else {
                format!("{} not found.", name)
            },
            Error::Parse(err) => format!("Parse error: {}", err),
            Error::Token(err) => format!("Parsing access token: {}", err),

            Error::Http(err) => format!("HTTP: {}", err),
            Error::Io(err) => format!("I/O: {}", err),
            Error::Json(err) => format!("Parsing JSON: {}", err),
            Error::Toml(err) => format!("Parsing TOML: {}", err),
            Error::Url(err) => format!("Parsing URL: {}", err),
            Error::Uuid(err) => format!("Parsing UUID: {}", err),
            Error::Zip(err) => format!("Zip I/O: {}", err),
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
