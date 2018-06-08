use failure::Error;
use reqwest::Client;
use serde_json::Value;
use std::path::PathBuf;
use uuid::Uuid;

use datatypes::Url;


pub trait Campaign {
    fn create(&self, campaign_id: Uuid, name: &str, groups: Vec<&str>) -> Result<(), Error>;
    fn get(&self, campaign_id: Uuid) -> Result<(), Error>;
    fn launch(&self, campaign_id: Uuid) -> Result<(), Error>;
    fn cancel(&self, campaign_id: Uuid) -> Result<(), Error>;
}

pub struct Manager {
    client: Client,
    remote: Url,
    credentials: PathBuf,
}

impl Manager {
    pub fn new(remote: Url, credentials: PathBuf) -> Self {
        Manager {
            client: Client::new(),
            remote,
            credentials,
        }
    }
}

impl Campaign for Manager {
    fn create(&self, campaign_id: Uuid, name: &str, groups: Vec<&str>) -> Result<(), Error> {
        let resp: Value = self
            .client
            .post(&format!("{}/api/v2/campaigns", self.remote))
            .json(&json!({"name": name, "update": campaign_id, "groups": groups}))
            .send()?
            .json()?;
        Ok(())
    }

    fn get(&self, campaign_id: Uuid) -> Result<(), Error> {
        let resp: Value = self
            .client
            .get(&format!("{}/api/v2/campaigns/{}", self.remote, campaign_id))
            .send()?
            .json()?;
        Ok(())
    }

    fn launch(&self, campaign_id: Uuid) -> Result<(), Error> {
        let resp: Value = self
            .client
            .post(&format!(
                "{}/api/v2/campaigns/{}/launch",
                self.remote, campaign_id
            ))
            .send()?
            .json()?;
        Ok(())
    }

    fn cancel(&self, campaign_id: Uuid) -> Result<(), Error> {
        let resp: Value = self
            .client
            .post(&format!(
                "{}/api/v2/campaigns/{}/cancel",
                self.remote, campaign_id
            ))
            .send()?
            .json()?;
        Ok(())
    }
}
