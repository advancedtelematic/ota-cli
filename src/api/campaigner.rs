use clap::ArgMatches;
use reqwest::{Client, Response};
use uuid::Uuid;

use config::Config;
use error::Result;
use http::{Http, HttpMethods};


/// Available Campaigner API methods.
pub trait CampaignerApi {
    fn create_campaign(&mut Config, update: Uuid, name: &str, groups: &[Uuid]) -> Result<Response>;
    fn launch_campaign(&mut Config, campaign: Uuid) -> Result<Response>;
    fn cancel_campaign(&mut Config, campaign: Uuid) -> Result<Response>;

    fn list_campaign_info(&mut Config, campaign: Uuid) -> Result<Response>;
    fn list_campaign_stats(&mut Config, campaign: Uuid) -> Result<Response>;
    fn list_all_campaigns(&mut Config) -> Result<Response>;
}

/// Make API calls to manage campaigns.
pub struct Campaigner;

impl<'a> Campaigner {
    /// Parse CLI arguments to create a new campaign.
    pub fn create_from_flags(config: &mut Config, flags: &ArgMatches<'a>) -> Result<Response> {
        let update = flags.value_of("update").expect("--update").parse()?;
        let name = flags.value_of("name").expect("--name");
        let groups = flags
            .values_of("groups")
            .expect("--groups")
            .map(Uuid::parse_str)
            .collect::<::std::result::Result<Vec<_>, _>>()?;
        Self::create_campaign(config, update, name, &groups)
    }

    /// Parse CLI arguments to list campaign information.
    pub fn list_from_flags(config: &mut Config, flags: &ArgMatches<'a>) -> Result<Response> {
        let campaign = || flags.value_of("campaign").expect("--campaign flag").parse();
        if flags.is_present("all") {
            Self::list_all_campaigns(config)
        } else if flags.is_present("stats") {
            Self::list_campaign_stats(config, campaign()?)
        } else {
            Self::list_campaign_info(config, campaign()?)
        }
    }
}

impl CampaignerApi for Campaigner {
    fn create_campaign(config: &mut Config, update: Uuid, name: &str, groups: &[Uuid]) -> Result<Response> {
        debug!("creating campaign {} with update {} for groups: {:?}", name, update, groups);
        let req = Client::new()
            .post(&format!("{}api/v2/campaigns", config.campaigner))
            .json(&json!({"update": format!("{}", update), "name": name, "groups": groups}))
            .build()?;
        Http::send(req, config.token()?)
    }

    fn launch_campaign(config: &mut Config, campaign: Uuid) -> Result<Response> {
        debug!("launching campaign {}", campaign);
        let req = Client::new()
            .post(&format!("{}api/v2/campaigns/{}/launch", config.campaigner, campaign))
            .build()?;
        Http::send(req, config.token()?)
    }

    fn cancel_campaign(config: &mut Config, campaign: Uuid) -> Result<Response> {
        debug!("cancelling campaign {}", campaign);
        Http::post(&format!("{}api/v2/campaigns/{}/cancel", config.campaigner, campaign), config.token()?)
    }

    fn list_campaign_info(config: &mut Config, campaign: Uuid) -> Result<Response> {
        debug!("getting info for campaign {}", campaign);
        Http::get(&format!("{}api/v2/campaigns/{}", config.campaigner, campaign), config.token()?)
    }

    fn list_campaign_stats(config: &mut Config, campaign: Uuid) -> Result<Response> {
        debug!("getting stats for campaign {}", campaign);
        Http::get(&format!("{}api/v2/campaigns/{}/stats", config.campaigner, campaign), config.token()?)
    }

    fn list_all_campaigns(config: &mut Config) -> Result<Response> {
        debug!("getting a list of campaigns");
        Http::get(&format!("{}api/v2/campaigns", config.campaigner), config.token()?)
    }
}
