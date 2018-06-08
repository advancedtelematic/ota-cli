use failure::Error;
use serde::{self, Deserialize, Deserializer};
use std::str::FromStr;


#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Action {
    Create,
    Launch,
    Get,
    Cancel,
}

impl FromStr for Action {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_lowercase().as_ref() {
            "create" => Ok(Action::Create),
            "launch" => Ok(Action::Launch),
            "get" => Ok(Action::Get),
            "cancel" => Ok(Action::Cancel),
            _ => bail!("unknown action: {}", s),
        }
    }
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let s: String = Deserialize::deserialize(de)?;
        s.parse()
            .map_err(|err| serde::de::Error::custom(format!("{}", err)))
    }
}
