use failure::Error;
use reqwest::header::{Authorization, Bearer, ContentType, Header};
use reqwest::Client;
use serde_json::Value;
use uuid::Uuid;

use datatype::{AccessToken, Url};

pub trait Campaign {
    fn create(&self, campaign_id: Uuid, name: &str, groups: Vec<&str>) -> Result<(), Error>;
    fn get(&self, campaign_id: Uuid) -> Result<(), Error>;
    fn launch(&self, campaign_id: Uuid) -> Result<(), Error>;
    fn stats(&self, campaign_id: Uuid) -> Result<(), Error>;
    fn cancel(&self, campaign_id: Uuid) -> Result<(), Error>;
}

pub struct Manager<'c> {
    client: &'c Client,
    campaigner: Url,
    token: AccessToken,
}

impl<'c> Manager<'c> {
    pub fn new(client: &'c Client, campaigner: Url, token: AccessToken) -> Self {
        Manager { client, campaigner, token }
    }

    fn bearer(&self) -> impl Header {
        Authorization(Bearer {
            token: self.token.access_token.clone(),
        })
    }
}

impl<'c> Campaign for Manager<'c> {
    fn create(&self, campaign_id: Uuid, name: &str, groups: Vec<&str>) -> Result<(), Error> {
        debug!("creating campaign - id: {}, name: {}, groups: {:?}", campaign_id, name, groups);
        let resp: Value = self.client
            .post(&format!("{}/api/v2/campaigns", self.campaigner))
            .header(self.bearer())
            .header(ContentType::json())
            .json(&json!({"name": name, "update": campaign_id, "groups": groups}))
            .send()?
            .json()?;
        debug!("resp: {}", resp);
        Ok(())
    }

    fn get(&self, campaign_id: Uuid) -> Result<(), Error> {
        debug!("getting campaign - id: {}", campaign_id);
        let resp: Value = self.client
            .get(&format!("{}/api/v2/campaigns/{}", self.campaigner, campaign_id))
            .header(self.bearer())
            .send()?
            .json()?;
        debug!("resp: {}", resp);
        Ok(())
    }

    fn launch(&self, campaign_id: Uuid) -> Result<(), Error> {
        debug!("launching campaign - id: {}", campaign_id);
        let resp: Value = self.client
            .post(&format!("{}/api/v2/campaigns/{}/launch", self.campaigner, campaign_id))
            .header(self.bearer())
            .send()?
            .json()?;
        debug!("resp: {}", resp);
        Ok(())
    }

    fn stats(&self, campaign_id: Uuid) -> Result<(), Error> {
        debug!("getting stats for campaign - id: {}", campaign_id);
        let resp: Value = self.client
            .get(&format!("{}/api/v2/campaigns/{}/stats", self.campaigner, campaign_id))
            .header(self.bearer())
            .send()?
            .json()?;
        debug!("resp: {}", resp);
        Ok(())
    }

    fn cancel(&self, campaign_id: Uuid) -> Result<(), Error> {
        debug!("cancelling campaign - id: {}", campaign_id);
        let resp: Value = self.client
            .post(&format!("{}/api/v2/campaigns/{}/cancel", self.campaigner, campaign_id))
            .header(self.bearer())
            .send()?
            .json()?;
        debug!("resp: {}", resp);
        Ok(())
    }
}
