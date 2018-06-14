use reqwest::header::{Authorization, Bearer, Header};
use reqwest::Client;
use uuid::Uuid;

use api::auth_plus::Token;
use error::Result;
use url::Url;
use util::print_json;

const API_VERSION: &'static str = "api/v2/campaigns";

/// Available campaigner API methods.
pub trait Campaigner {
    fn create(&self, campaign_id: Uuid, name: &str, groups: &[Uuid]) -> Result<()>;
    fn get(&self, campaign_id: Uuid) -> Result<()>;
    fn launch(&self, campaign_id: Uuid) -> Result<()>;
    fn stats(&self, campaign_id: Uuid) -> Result<()>;
    fn cancel(&self, campaign_id: Uuid) -> Result<()>;
}

/// Handle API calls to the campaigner server.
pub struct CampaignHandler<'c> {
    client: &'c Client,
    server: Url,
    token: Token,
}

impl<'c> CampaignHandler<'c> {
    pub fn new(client: &'c Client, server: Url, token: Token) -> Self {
        CampaignHandler { client, server, token }
    }

    fn bearer(&self) -> Result<impl Header> {
        Ok(Authorization(Bearer {
            token: (self.token)()?.access_token,
        }))
    }
}

impl<'c> Campaigner for CampaignHandler<'c> {
    fn create(&self, campaign_id: Uuid, name: &str, groups: &[Uuid]) -> Result<()> {
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

    fn get(&self, campaign_id: Uuid) -> Result<()> {
        debug!("getting campaign with id: {}", campaign_id);
        let resp = self.client
            .get(&format!("{}{}/{}", self.server, API_VERSION, campaign_id))
            .header(self.bearer()?)
            .send()?
            .json()?;
        print_json(resp)
    }

    fn launch(&self, campaign_id: Uuid) -> Result<()> {
        debug!("launching campaign with id: {}", campaign_id);
        let resp = self.client
            .post(&format!("{}{}/{}/launch", self.server, API_VERSION, campaign_id))
            .header(self.bearer()?)
            .send()?
            .json()?;
        print_json(resp)
    }

    fn stats(&self, campaign_id: Uuid) -> Result<()> {
        debug!("getting stats for campaign with id: {}", campaign_id);
        let resp = self.client
            .get(&format!("{}{}/{}/stats", self.server, API_VERSION, campaign_id))
            .header(self.bearer()?)
            .send()?
            .json()?;
        print_json(resp)
    }

    fn cancel(&self, campaign_id: Uuid) -> Result<()> {
        debug!("cancelling campaign with id: {}", campaign_id);
        let resp = self.client
            .post(&format!("{}{}/{}/cancel", self.server, API_VERSION, campaign_id))
            .header(self.bearer()?)
            .send()?
            .json()?;
        print_json(resp)
    }
}
