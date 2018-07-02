use clap::ArgMatches;
use uuid::Uuid;

use api::director::{Director, DirectorApi, HardwareTargets, UpdateTargets};
use config::Config;
use error::{Error, Result};
use util::print_resp;


/// Available Campaigner API methods.
pub trait CampaignerApi {
    fn create(&mut Config, campaign_id: Uuid, name: &str, groups: &[Uuid]) -> Result<()>;
    fn get(&mut Config, campaign_id: Uuid) -> Result<()>;
    fn launch(&mut Config, campaign_id: Uuid) -> Result<()>;
    fn stats(&mut Config, campaign_id: Uuid) -> Result<()>;
    fn cancel(&mut Config, campaign_id: Uuid) -> Result<()>;
}

/// Make API calls to manage campaigns.
pub struct Campaigner;

impl<'a> Campaigner {
    /// Parse CLI arguments to create a new campaign.
    pub fn create_from_matches(mut config: &mut Config, matches: &ArgMatches<'a>) -> Result<()> {
        let name = matches.value_of("name").expect("--name");
        let groups = matches
            .values_of("groups")
            .expect("--groups")
            .map(Uuid::parse_str)
            .collect::<::std::result::Result<Vec<_>, _>>()?;
        let targets_file = matches.value_of("targets").expect("--targets");
        let targets = UpdateTargets::from(HardwareTargets::from_file(targets_file)?);
        let campaign_id = Director::create_mtu(&mut config, &targets)?;
        Self::create(config, campaign_id, name, &groups)
    }
}

impl CampaignerApi for Campaigner {
    fn create(config: &mut Config, campaign_id: Uuid, name: &str, groups: &[Uuid]) -> Result<()> {
        let url = format!("{}api/v2/campaigns", config.campaigner);
        debug!("creating campaign {} ({}) for groups: {:?}", campaign_id, name, groups);
        config
            .client()
            .post(&url)
            .json(&json!({"name": name, "update": campaign_id, "groups": groups}))
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn get(config: &mut Config, campaign_id: Uuid) -> Result<()> {
        let url = format!("{}api/v2/campaigns/{}", config.campaigner, campaign_id);
        debug!("getting campaign {}", campaign_id);
        config
            .client()
            .get(&url)
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn launch(config: &mut Config, campaign_id: Uuid) -> Result<()> {
        let url = format!("{}api/v2/campaigns/{}/launch", config.campaigner, campaign_id);
        debug!("launching campaign {}", campaign_id);
        config
            .client()
            .post(&url)
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn stats(config: &mut Config, campaign_id: Uuid) -> Result<()> {
        let url = format!("{}api/v2/campaigns/{}/stats", config.campaigner, campaign_id);
        debug!("getting stats for campaign {}", campaign_id);
        config
            .client()
            .get(&url)
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn cancel(config: &mut Config, campaign_id: Uuid) -> Result<()> {
        let url = format!("{}api/v2/campaigns/{}/cancel", config.campaigner, campaign_id);
        debug!("cancelling campaign {}", campaign_id);
        config
            .client()
            .post(&url)
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }
}
