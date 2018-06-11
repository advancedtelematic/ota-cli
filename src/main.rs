#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate toml;
extern crate url;
extern crate uuid;
extern crate zip;

mod campaign;
mod config;
mod datatype;

use clap::{AppSettings, ArgMatches};
use env_logger::Builder;
use failure::Error;
use log::LevelFilter;
use reqwest::Client;
use std::io::Write;
use uuid::Uuid;

use campaign::{Campaign, Manager};
use datatype::AccessToken;

fn main() -> Result<(), Error> {
    let args = parse_args();
    start_logging(args.value_of("log-level").unwrap_or("INFO"));

    let client = Client::new();
    let credentials = args.value_of("credentials-zip").unwrap().into();
    let token = AccessToken::refresh(&client, credentials)?;

    let campaigner = args.value_of("campaigner").unwrap().parse()?;
    let manager = Manager::new(&client, campaigner, token);

    match args.subcommand() {
        ("create", Some(args)) => {
            let campaign_id = match args.value_of("campaign-id") {
                Some(id) => id.parse()?,
                None => Uuid::new_v4(),
            };
            let name = args.value_of("name").unwrap();
            let groups: Vec<_> = args.values_of("groups").unwrap().collect();
            manager.create(campaign_id, name, groups)?;
        }

        ("get", Some(args)) => {
            let campaign_id: Uuid = args.value_of("campaign-id").unwrap().parse()?;
            manager.get(campaign_id)?;
        }

        ("launch", Some(args)) => {
            let campaign_id: Uuid = args.value_of("campaign-id").unwrap().parse()?;
            manager.launch(campaign_id)?;
        }

        ("stats", Some(args)) => {
            let campaign_id: Uuid = args.value_of("campaign-id").unwrap().parse()?;
            manager.stats(campaign_id)?;
        }

        ("cancel", Some(args)) => {
            let campaign_id: Uuid = args.value_of("campaign-id").unwrap().parse()?;
            manager.cancel(campaign_id)?;
        }

        _ => unreachable!(),
    }

    Ok(())
}

fn parse_args<'a>() -> ArgMatches<'a> {
    clap_app!(("campaign-manager") =>
        (version: crate_version!())
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (setting: AppSettings::VersionlessSubcommands)
        (setting: AppSettings::DeriveDisplayOrder)
        (setting: AppSettings::DisableHelpSubcommand)

        (@arg ("log-level"): -l --("log-level") +takes_value +global "(optional) Set the logging level")
        (@arg campaigner: -c --campaigner +takes_value +global "Campaigner URL")
        (@arg ("credentials-zip"): -z --("credentials-zip") +takes_value +global "Path to credentials.zip")

        (@subcommand create =>
            (about: "Create a new campaign")
            (setting: AppSettings::ArgRequiredElseHelp)
            (setting: AppSettings::DeriveDisplayOrder)
            (@arg ("campaign-id"): -i --("campaign-id") [uuid] "(optional) Specify the campaign ID")
            (@arg name: -n --name <name> "The campaign name")
            (@arg groups: -g --groups <group> ... "Apply the campaign to the following groups")
        )

        (@subcommand get =>
            (about: "Retrieve campaign information")
            (setting: AppSettings::ArgRequiredElseHelp)
            (setting: AppSettings::DeriveDisplayOrder)
            (@arg ("campaign-id"): -i --("campaign-id") <uuid> "The campaign ID")
        )

        (@subcommand launch =>
            (about: "Launch a created campaign")
            (setting: AppSettings::ArgRequiredElseHelp)
            (setting: AppSettings::DeriveDisplayOrder)
            (@arg ("campaign-id"): -i --("campaign-id") <uuid> "The campaign ID")
        )

        (@subcommand stats =>
            (about: "Retrieve stats from a campaign")
            (setting: AppSettings::ArgRequiredElseHelp)
            (setting: AppSettings::DeriveDisplayOrder)
            (@arg ("campaign-id"): -i --("campaign-id") <uuid> "The campaign ID")
        )

        (@subcommand cancel =>
            (about: "Cancel a launched campaign")
            (setting: AppSettings::ArgRequiredElseHelp)
            (setting: AppSettings::DeriveDisplayOrder)
            (@arg ("campaign-id"): -i --("campaign-id") <uuid> "The campaign ID")
        )
    ).get_matches()
}

fn start_logging(level: &str) {
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .parse(level);
    if level != "TRACE" {
        builder.filter(Some("tokio"), LevelFilter::Info);
        builder.filter(Some("hyper"), LevelFilter::Info);
    }
    builder.init();
}
