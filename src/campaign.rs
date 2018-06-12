use reqwest::header::{Authorization, Bearer, Header};
use reqwest::Client;
use serde_json::{self, Value};
use std::io::{self, Write};
use uuid::Uuid;

use datatype::{Token, Url};
use error::Error;

const API_VERSION: &'static str = "api/v2/campaigns";

pub trait Campaign {
    fn create(&self, campaign_id: Uuid, name: &str, groups: Vec<Uuid>) -> Result<(), Error>;
    fn get(&self, campaign_id: Uuid) -> Result<(), Error>;
    fn launch(&self, campaign_id: Uuid) -> Result<(), Error>;
    fn stats(&self, campaign_id: Uuid) -> Result<(), Error>;
    fn cancel(&self, campaign_id: Uuid) -> Result<(), Error>;
}

pub struct Manager<'c> {
    client: &'c Client,
    campaigner: Url,
    token: Token,
}

impl<'c> Manager<'c> {
    pub fn new(client: &'c Client, campaigner: Url, token: Token) -> Self {
        Manager {
            client,
            campaigner,
            token,
        }
    }

    fn bearer(&self) -> Result<impl Header, Error> {
        Ok(Authorization(Bearer {
            token: (self.token)()?.access_token,
        }))
    }
}

impl<'c> Campaign for Manager<'c> {
    fn create(&self, campaign_id: Uuid, name: &str, groups: Vec<Uuid>) -> Result<(), Error> {
        debug!(
            "creating campaign with id: {}, name: {}, groups: {:?}",
            campaign_id, name, groups
        );
        let resp: Value = self.client
            .post(&format!("{}{}", self.campaigner, API_VERSION))
            .header(self.bearer()?)
            .json(&json!({"name": name, "update": campaign_id, "groups": groups}))
            .send()?
            .json()?;
        print(resp)
    }

    fn get(&self, campaign_id: Uuid) -> Result<(), Error> {
        debug!("getting campaign with id: {}", campaign_id);
        let resp: Value = self.client
            .get(&format!("{}{}/{}", self.campaigner, API_VERSION, campaign_id))
            .header(self.bearer()?)
            .send()?
            .json()?;
        print(resp)
    }

    fn launch(&self, campaign_id: Uuid) -> Result<(), Error> {
        debug!("launching campaign with id: {}", campaign_id);
        let resp: Value = self.client
            .post(&format!("{}{}/{}/launch", self.campaigner, API_VERSION, campaign_id))
            .header(self.bearer()?)
            .send()?
            .json()?;
        print(resp)
    }

    fn stats(&self, campaign_id: Uuid) -> Result<(), Error> {
        debug!("getting stats for campaign with id: {}", campaign_id);
        let resp: Value = self.client
            .get(&format!("{}{}/{}/stats", self.campaigner, API_VERSION, campaign_id))
            .header(self.bearer()?)
            .send()?
            .json()?;
        print(resp)
    }

    fn cancel(&self, campaign_id: Uuid) -> Result<(), Error> {
        debug!("cancelling campaign with id: {}", campaign_id);
        let resp: Value = self.client
            .post(&format!("{}{}/{}/cancel", self.campaigner, API_VERSION, campaign_id))
            .header(self.bearer()?)
            .send()?
            .json()?;
        print(resp)
    }
}

fn print(resp: Value) -> Result<(), Error> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write(serde_json::to_string_pretty(&resp)?.as_bytes());
    Ok(())
}
