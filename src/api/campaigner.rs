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

    fn list_updates(&mut Config) -> Result<Response>;
    fn create_update(&mut Config, update: Uuid, name: &str, description: &str) -> Result<Response>;

    fn list_campaign_info(&mut Config, campaign: Uuid) -> Result<Response>;
    fn list_campaign_stats(&mut Config, campaign: Uuid) -> Result<Response>;
    fn list_all_campaigns(&mut Config) -> Result<Response>;
}

/// Make API calls to manage campaigns.
pub struct Campaigner;

impl<'a> Campaigner {
    /// Parse CLI arguments to create a new campaign.
    pub fn create_from_args(config: &mut Config, args: &ArgMatches<'a>) -> Result<Response> {
        let update = args.value_of("update").expect("--update").parse()?;
        let name = args.value_of("name").expect("--name");
        let groups = args
            .values_of("groups")
            .expect("--groups")
            .map(Uuid::parse_str)
            .collect::<::std::result::Result<Vec<_>, _>>()?;
        Self::create_campaign(config, update, name, &groups)
    }

    /// Parse CLI arguments to list campaign information.
    pub fn list_from_args(config: &mut Config, args: &ArgMatches<'a>) -> Result<Response> {
        let campaign = || args.value_of("campaign").expect("--campaign flag").parse();
        if args.is_present("all") {
            Self::list_all_campaigns(config)
        } else if args.is_present("stats") {
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
            .json(&json!({"update": format!("{}", update), "name": name, "groups": groups}));
        Http::send(req, config.token()?)
    }

    fn launch_campaign(config: &mut Config, campaign: Uuid) -> Result<Response> {
        debug!("launching campaign {}", campaign);
        let req = Client::new().post(&format!("{}api/v2/campaigns/{}/launch", config.campaigner, campaign));
        Http::send(req, config.token()?)
    }

    fn cancel_campaign(config: &mut Config, campaign: Uuid) -> Result<Response> {
        debug!("cancelling campaign {}", campaign);
        Http::post(
            &format!("{}api/v2/campaigns/{}/cancel", config.campaigner, campaign),
            config.token()?,
        )
    }

    fn list_updates(config: &mut Config) -> Result<Response> {
        debug!("getting list of campaigner updates ");
        Http::get(&format!("{}api/v2/updates", config.campaigner), config.token()?)
    }

    fn create_update(config: &mut Config, update: Uuid, name: &str, description: &str) -> Result<Response> {
        debug!("creating update ");

        let req = Client::new()
            .post(&format!("{}api/v2/updates", config.campaigner))
            .json(&json!({"name": name, "description": description, "updateSource": {"id": format!("{}", update), "sourceType": "multi_target" }} ));

        Http::send(req, config.token()?)
    }

    fn list_campaign_info(config: &mut Config, campaign: Uuid) -> Result<Response> {
        debug!("getting info for campaign {}", campaign);
        Http::get(&format!("{}api/v2/campaigns/{}", config.campaigner, campaign), config.token()?)
    }

    fn list_campaign_stats(config: &mut Config, campaign: Uuid) -> Result<Response> {
        debug!("getting stats for campaign {}", campaign);
        Http::get(
            &format!("{}api/v2/campaigns/{}/stats", config.campaigner, campaign),
            config.token()?,
        )
    }

    fn list_all_campaigns(config: &mut Config) -> Result<Response> {
        debug!("getting a list of campaigns");
        Http::get(&format!("{}api/v2/campaigns", config.campaigner), config.token()?)
    }
}
