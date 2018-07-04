use clap::ArgMatches;
use std::result;
use uuid::Uuid;

use config::Config;
use error::{Error, Result};
use util::print_resp;


/// Available Campaigner API methods.
pub trait CampaignerApi {
    fn create(&mut Config, update: Uuid, name: &str, groups: &[Uuid]) -> Result<()>;
    fn get(&mut Config, campaign: Uuid) -> Result<()>;
    fn launch(&mut Config, campaign: Uuid) -> Result<()>;
    fn stats(&mut Config, campaign: Uuid) -> Result<()>;
    fn cancel(&mut Config, campaign: Uuid) -> Result<()>;
}

/// Make API calls to manage campaigns.
pub struct Campaigner;

impl<'a> Campaigner {
    /// Parse CLI arguments to create a new campaign.
    pub fn create_from_flags(config: &mut Config, flags: &ArgMatches<'a>) -> Result<()> {
        let update = flags.value_of("update").expect("--update").parse()?;
        let name = flags.value_of("name").expect("--name");
        let groups = flags
            .values_of("groups")
            .expect("--groups")
            .map(Uuid::parse_str)
            .collect::<result::Result<Vec<_>, _>>()?;
        Self::create(config, update, name, &groups)
    }
}

impl CampaignerApi for Campaigner {
    fn create(config: &mut Config, update: Uuid, name: &str, groups: &[Uuid]) -> Result<()> {
        debug!("creating campaign {} with update {} for groups: {:?}", name, update, groups);
        config
            .client()
            .post(&format!("{}api/v2/campaigns", config.campaigner))
            .json(&json!({"update": format!("{}", update), "name": name, "groups": groups}))
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn get(config: &mut Config, campaign: Uuid) -> Result<()> {
        debug!("getting campaign {}", campaign);
        config
            .client()
            .get(&format!("{}api/v2/campaigns/{}", config.campaigner, campaign))
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn launch(config: &mut Config, campaign: Uuid) -> Result<()> {
        debug!("launching campaign {}", campaign);
        config
            .client()
            .post(&format!("{}api/v2/campaigns/{}/launch", config.campaigner, campaign))
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn stats(config: &mut Config, campaign: Uuid) -> Result<()> {
        debug!("getting stats for campaign {}", campaign);
        config
            .client()
            .get(&format!("{}api/v2/campaigns/{}/stats", config.campaigner, campaign))
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn cancel(config: &mut Config, campaign: Uuid) -> Result<()> {
        debug!("cancelling campaign {}", campaign);
        config
            .client()
            .post(&format!("{}api/v2/campaigns/{}/cancel", config.campaigner, campaign))
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }
}
