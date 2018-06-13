#[macro_use]
extern crate clap;
extern crate env_logger;
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
extern crate url_serde;
extern crate uuid;
extern crate zip;

mod campaigner;
mod director;
mod error;
mod token;
mod util;

use clap::{AppSettings, ArgMatches};
use env_logger::Builder;
use log::LevelFilter;
use reqwest::Client;
use std::{io::Write, path::PathBuf};
use uuid::Uuid;

use campaigner::{Action, Campaign, Campaigner};
use error::Error;
use token::AccessToken;

fn main() -> Result<(), Error> {
    let args = parse_args();
    start_logging(args.value_of("log-level").unwrap_or("INFO"));

    let (cmd, sub) = args.subcommand();
    let action = cmd.parse()?;
    let sub = sub.expect("subcommand matches");

    let client = Client::new();
    let clone = client.clone();
    let zip_path: PathBuf = args.value_of("credentials-zip").expect("--credentials-zip").into();
    let token = Box::new(move || AccessToken::refresh(&clone, &zip_path));

    let url = args.value_of("campaigner-url").expect("--campaigner-url").parse()?;
    let campaigner = Campaigner::new(&client, url, token);
    let campaign_id = |args: &ArgMatches| args.value_of("campaign-id").expect("--campaign-id").parse::<Uuid>();

    match action {
        Action::Create => {
            let campaign_id = match sub.value_of("campaign-id") {
                Some(id) => id.parse()?,
                None => Uuid::new_v4(),
            };
            let name = sub.value_of("name").expect("--name");
            let groups = sub.values_of("groups")
                .expect("--groups")
                .map(Uuid::parse_str)
                .collect::<Result<Vec<_>, _>>()?;
            campaigner.create(campaign_id, name, &groups)
        }
        Action::Get => campaigner.get(campaign_id(sub)?),
        Action::Launch => campaigner.launch(campaign_id(sub)?),
        Action::Stats => campaigner.stats(campaign_id(sub)?),
        Action::Cancel => campaigner.cancel(campaign_id(sub)?),
    }
}

fn parse_args<'a>() -> ArgMatches<'a> {
    clap_app!(("ota-cli") =>
        (version: crate_version!())
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (setting: AppSettings::VersionlessSubcommands)
        (setting: AppSettings::DeriveDisplayOrder)
        (setting: AppSettings::DisableHelpSubcommand)

        (@arg ("log-level"): -l --("log-level") +takes_value +global "(optional) Set the logging level")
        (@arg ("credentials-zip"): -z --("credentials-zip") +takes_value +global "Path to credentials.zip")
        (@arg ("campaigner-url"): -c --("campaigner-url") +takes_value +global "Campaigner server")

        (@subcommand create =>
            (about: "Create a new campaign")
            (setting: AppSettings::ArgRequiredElseHelp)
            (setting: AppSettings::DeriveDisplayOrder)
            (@arg ("campaign-id"): -i --("campaign-id") [uuid] "(optional) Specify the campaign ID")
            (@arg name: -n --name <name> "The campaign name")
            (@arg groups: -g --groups <uuid> ... "Apply the campaign to the following groups")
            (@arg ("director-url"): -d --("director-url") <url> "Director server")
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
    if level.to_uppercase() != "TRACE" {
        builder.filter(Some("tokio"), LevelFilter::Info);
        builder.filter(Some("hyper"), LevelFilter::Info);
    }
    builder.init();
}
