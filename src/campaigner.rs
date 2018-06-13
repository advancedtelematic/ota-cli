use reqwest::header::{Authorization, Bearer, Header};
use reqwest::Client;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt::{self, Display, Formatter},
          str::FromStr};
use uuid::Uuid;

use error::Error;
use token::Token;
use url::Url;
use util::print_json;

const API_VERSION: &'static str = "api/v2/campaigns";

/// Available campaigner API methods.
pub trait Campaign {
    fn create(&self, campaign_id: Uuid, name: &str, groups: &[Uuid]) -> Result<(), Error>;
    fn get(&self, campaign_id: Uuid) -> Result<(), Error>;
    fn launch(&self, campaign_id: Uuid) -> Result<(), Error>;
    fn stats(&self, campaign_id: Uuid) -> Result<(), Error>;
    fn cancel(&self, campaign_id: Uuid) -> Result<(), Error>;
}

/// Manage API calls to the remote campaigner server.
pub struct Campaigner<'c> {
    client: &'c Client,
    server: Url,
    token: Token,
}

impl<'c> Campaigner<'c> {
    pub fn new(client: &'c Client, server: Url, token: Token) -> Self {
        Campaigner { client, server, token }
    }

    fn bearer(&self) -> Result<impl Header, Error> {
        Ok(Authorization(Bearer {
            token: (self.token)()?.access_token,
        }))
    }
}

impl<'c> Campaign for Campaigner<'c> {
    fn create(&self, campaign_id: Uuid, name: &str, groups: &[Uuid]) -> Result<(), Error> {
        debug!(
            "creating campaign with id: {}, name: {}, groups: {:?}",
            campaign_id, name, groups
        );
        let resp = self.client
            .post(&format!("{}{}", self.server, API_VERSION))
            .header(self.bearer()?)
            .json(&json!({"name": name, "update": campaign_id, "groups": groups}))
            .send()?
            .json()?;
        print_json(resp)
    }

    fn get(&self, campaign_id: Uuid) -> Result<(), Error> {
        debug!("getting campaign with id: {}", campaign_id);
        let resp = self.client
            .get(&format!("{}{}/{}", self.server, API_VERSION, campaign_id))
            .header(self.bearer()?)
            .send()?
            .json()?;
        print_json(resp)
    }

    fn launch(&self, campaign_id: Uuid) -> Result<(), Error> {
        debug!("launching campaign with id: {}", campaign_id);
        let resp = self.client
            .post(&format!("{}{}/{}/launch", self.server, API_VERSION, campaign_id))
            .header(self.bearer()?)
            .send()?
            .json()?;
        print_json(resp)
    }

    fn stats(&self, campaign_id: Uuid) -> Result<(), Error> {
        debug!("getting stats for campaign with id: {}", campaign_id);
        let resp = self.client
            .get(&format!("{}{}/{}/stats", self.server, API_VERSION, campaign_id))
            .header(self.bearer()?)
            .send()?
            .json()?;
        print_json(resp)
    }

    fn cancel(&self, campaign_id: Uuid) -> Result<(), Error> {
        debug!("cancelling campaign with id: {}", campaign_id);
        let resp = self.client
            .post(&format!("{}{}/{}/cancel", self.server, API_VERSION, campaign_id))
            .header(self.bearer()?)
            .send()?
            .json()?;
        print_json(resp)
    }
}

/// Available campaigner API actions.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Action {
    Create,
    Launch,
    Get,
    Stats,
    Cancel,
}

impl FromStr for Action {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_lowercase().as_ref() {
            "create" => Ok(Action::Create),
            "launch" => Ok(Action::Launch),
            "get" => Ok(Action::Get),
            "stats" => Ok(Action::Stats),
            "cancel" => Ok(Action::Cancel),
            _ => Err(Error::Action(s.into())),
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Action::Create => write!(f, "create"),
            Action::Launch => write!(f, "launch"),
            Action::Get => write!(f, "get"),
            Action::Stats => write!(f, "stats"),
            Action::Cancel => write!(f, "cancel"),
        }
    }
}

impl Serialize for Action {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&format!("{}", self))
    }
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let s: String = Deserialize::deserialize(de)?;
        s.parse().map_err(|err| serde::de::Error::custom(format!("{}", err)))
    }
}
