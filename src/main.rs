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

mod api;
mod commands;
mod error;
mod util;

use clap::{AppSettings, ArgMatches};
use env_logger::Builder;
use log::LevelFilter;
use reqwest::Client;
use std::{io::Write, path::PathBuf};
use uuid::Uuid;

use api::auth_plus::AccessToken;
use api::campaigner::{CampaignHandler, Campaigner};
use api::director::{Director, DirectorHandler};
use commands::{Campaign, Command, Package};
use error::Error;

fn main() -> Result<(), Error> {
    let args = parse_args();
    start_logging(args.value_of("log-level").unwrap_or("INFO"));

    let (cmd, sub) = args.subcommand();
    let cmd = cmd.parse::<Command>()?;
    let sub = sub.expect("subcommand matches");

    let client = Client::new();
    let clone = client.clone();
    let zip_path: PathBuf = args.value_of("credentials-zip").expect("--credentials-zip").into();
    let token = Box::new(move || AccessToken::refresh(&clone, &zip_path));

    match cmd {
        Command::Campaign => {
            let (cmd, sub) = sub.subcommand();
            let cmd = cmd.parse::<Campaign>()?;
            let sub = sub.expect("campaign subcommand matches");

            let url = args.value_of("campaigner-url").expect("--campaigner-url").parse()?;
            let campaign = CampaignHandler::new(&client, url, token);
            let id = |args: &ArgMatches| args.value_of("campaign-id").expect("--campaign-id").parse::<Uuid>();

            match cmd {
                Campaign::Create => {
                    let campaign_id = match sub.value_of("campaign-id") {
                        Some(id) => id.parse()?,
                        None => Uuid::new_v4(),
                    };
                    let name = sub.value_of("name").expect("--name");
                    let groups = sub.values_of("groups")
                        .expect("--groups")
                        .map(Uuid::parse_str)
                        .collect::<Result<Vec<_>, _>>()?;
                    campaign.create(campaign_id, name, &groups)
                }
                Campaign::Get => campaign.get(id(sub)?),
                Campaign::Launch => campaign.launch(id(sub)?),
                Campaign::Stats => campaign.stats(id(sub)?),
                Campaign::Cancel => campaign.cancel(id(sub)?),
            }
        }

        Command::Package => {
            let (cmd, sub) = sub.subcommand();
            let cmd = cmd.parse::<Package>()?;
            let sub = sub.expect("package subcommand matches");

            match cmd {
                Package::Add => {
                    let name = sub.value_of("name").expect("--name");
                    let path = sub.value_of("path").expect("--path");
                    let version = sub.value_of("version").expect("--version");
                    let format = sub.value_of("format").expect("--format");
                    let hardware_ids: Vec<_> = sub.values_of("hardware-ids").expect("--hardware-ids").collect();
                    unimplemented!()
                }
            }
        }
    }
}

fn parse_args<'a>() -> ArgMatches<'a> {
    clap_app!(("ota-cli") =>
      (version: crate_version!())
      (setting: AppSettings::SubcommandRequiredElseHelp)
      (setting: AppSettings::VersionlessSubcommands)
      (setting: AppSettings::InferSubcommands)
      (setting: AppSettings::DeriveDisplayOrder)

      (@arg ("log-level"): -l --("log-level") +takes_value +global "(optional) Set the logging level")
      (@arg ("credentials-zip"): -z --("credentials-zip") +takes_value +global "Path to credentials.zip")

      (@subcommand campaign =>
        (about: "Manage OTA campaigns")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (setting: AppSettings::DeriveDisplayOrder)
        (@arg ("campaigner-url"): -c --("campaigner-url") +takes_value +global "Campaigner server")

        (@subcommand create =>
          (about: "Create a new campaign")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (@arg ("director-url"): -d --("director-url") <url> "Director server")
          (@arg ("campaign-id"): -i --("campaign-id") [uuid] "(optional) Specify the campaign ID")
          (@arg name: -n --name <name> "The campaign name")
          (@arg groups: -g --groups <uuid> ... "Apply the campaign to the following groups")
        )

        (@subcommand get =>
          (about: "Retrieve campaign information")
          (setting: AppSettings::ArgRequiredElseHelp)
          (@arg ("campaign-id"): -i --("campaign-id") <uuid> "The campaign ID")
        )

        (@subcommand launch =>
          (about: "Launch a created campaign")
          (setting: AppSettings::ArgRequiredElseHelp)
          (@arg ("campaign-id"): -i --("campaign-id") <uuid> "The campaign ID")
        )

        (@subcommand stats =>
          (about: "Retrieve stats from a campaign")
          (setting: AppSettings::ArgRequiredElseHelp)
          (@arg ("campaign-id"): -i --("campaign-id") <uuid> "The campaign ID")
        )

        (@subcommand cancel =>
          (about: "Cancel a launched campaign")
          (setting: AppSettings::ArgRequiredElseHelp)
          (@arg ("campaign-id"): -i --("campaign-id") <uuid> "The campaign ID")
        )
      )

      (@subcommand package =>
        (about: "Manage OTA packages")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (setting: AppSettings::DeriveDisplayOrder)

        (@subcommand add =>
          (about: "Add a new package")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (@arg name: -n --name <name> "The package name")
          (@arg version: -v --version <version> "The package version")
          (@arg format: -f --format <format> "Package format (binary or ostree)")
          (@arg path: -p --path [path] "Path to binary package")
          (@arg ("hardware-ids"): -h --("hardware-ids") <id> ... "Package works on these hardware IDs")
        )
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
