use failure::Error;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{self, Display, Formatter}, ops::Deref, str::FromStr,
};
use url;


/// Encapsulate a url with additional methods and traits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Url(pub url::Url);

impl Url {
    /// Append the string suffix to this URL, trimming multiple slashes.
    /// Will panic on parse failure.
    pub fn join(&self, suffix: &str) -> Url {
        let url = match (self.0.as_str().ends_with('/'), suffix.starts_with('/')) {
            (true, true) => format!("{}{}", self.0, suffix.trim_left_matches('/')),
            (false, false) => format!("{}/{}", self.0, suffix),
            _ => format!("{}{}", self.0, suffix),
        };
        Url(url::Url::parse(&url).expect(&format!("couldn't join `{}` with `{}`", self.0, suffix)))
    }
}

impl FromStr for Url {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(Url(url::Url::parse(s)?)) }
}

impl Serialize for Url {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&format!("{}", self))
    }
}

impl<'de> Deserialize<'de> for Url {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Url, D::Error> {
        let s: String = Deserialize::deserialize(de)?;
        s.parse()
            .map_err(|err| serde::de::Error::custom(format!("invalid url: {}", err)))
    }
}

impl Deref for Url {
    type Target = url::Url;

    fn deref(&self) -> &url::Url { &self.0 }
}

impl Display for Url {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let host = self.0.host_str().unwrap_or("localhost");
        if let Some(port) = self.0.port() {
            write!(
                f,
                "{}://{}:{}{}",
                self.0.scheme(),
                host,
                port,
                self.0.path()
            )
        } else {
            write!(f, "{}://{}{}", self.0.scheme(), host, self.0.path())
        }
    }
}
