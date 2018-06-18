use reqwest::header::{Authorization, Bearer, Headers};
use reqwest::Client;
use uuid::Uuid;

use api::auth_plus::Token;
use error::{Error, Result};
use url::Url;
use util::print_resp;

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

    fn auth(&self) -> Result<Headers> {
        let mut headers = Headers::new();
        if let Some(resp) = (self.token)()? {
            headers.set(Authorization(Bearer {
                token: resp.access_token,
            }));
        }
        Ok(headers)
    }
}

impl<'c> Campaigner for CampaignHandler<'c> {
    fn create(&self, campaign_id: Uuid, name: &str, groups: &[Uuid]) -> Result<()> {
        debug!(
            "creating campaign with id: {}, name: {}, groups: {:?}",
            campaign_id, name, groups
        );
        self.client
            .post(&format!("{}{}", self.server, API_VERSION))
            .json(&json!({"name": name, "update": campaign_id, "groups": groups}))
            .headers(self.auth()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn get(&self, campaign_id: Uuid) -> Result<()> {
        debug!("getting campaign with id: {}", campaign_id);
        self.client
            .get(&format!("{}{}/{}", self.server, API_VERSION, campaign_id))
            .headers(self.auth()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn launch(&self, campaign_id: Uuid) -> Result<()> {
        debug!("launching campaign with id: {}", campaign_id);
        self.client
            .post(&format!("{}{}/{}/launch", self.server, API_VERSION, campaign_id))
            .headers(self.auth()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn stats(&self, campaign_id: Uuid) -> Result<()> {
        debug!("getting stats for campaign with id: {}", campaign_id);
        self.client
            .get(&format!("{}{}/{}/stats", self.server, API_VERSION, campaign_id))
            .headers(self.auth()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn cancel(&self, campaign_id: Uuid) -> Result<()> {
        debug!("cancelling campaign with id: {}", campaign_id);
        self.client
            .post(&format!("{}{}/{}/cancel", self.server, API_VERSION, campaign_id))
            .headers(self.auth()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }
}
