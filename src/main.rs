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
extern crate serde_json as json;
extern crate toml;
extern crate uuid;

mod campaign;
mod config;

use clap::{AppSettings, ArgMatches};
use env_logger::Builder;
use failure::Error;
use log::LevelFilter;
use std::io::Write;
use uuid::Uuid;

use campaign::{Campaign, Manager};


fn main() -> Result<(), Error> {
    let args = parse_args();
    start_logging(args.value_of("log-level").unwrap_or("INFO"));

    match args.subcommand() {
        ("create", Some(args)) => {
            let manager = Manager::new(args.value_of("credentials").unwrap().into());
            let campaign_id = match args.value_of("campaign-id") {
                Some(id) => id.parse()?,
                None => Uuid::new_v4(),
            };
            let name = args.value_of("name").unwrap();
            let groups: Vec<_> = args.values_of("groups").unwrap().collect();
            manager.create(campaign_id, name, groups)?;
        }

        ("get", Some(args)) => {
            let manager = Manager::new(args.value_of("credentials").unwrap().into());
            let campaign_id: Uuid = args.value_of("campaign-id").unwrap().parse()?;
            manager.get(campaign_id)?;
        }

        ("launch", Some(args)) => {
            let manager = Manager::new(args.value_of("credentials").unwrap().into());
            let campaign_id: Uuid = args.value_of("campaign-id").unwrap().parse()?;
            manager.launch(campaign_id)?;
        }

        ("cancel", Some(args)) => {
            let manager = Manager::new(args.value_of("credentials").unwrap().into());
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

        (@arg ("log-level"): -l --("log-level") +takes_value +global "Set the logging level")

        (@subcommand create =>
            (about: "Create a new campaign")
            (setting: AppSettings::ArgRequiredElseHelp)
            (setting: AppSettings::DeriveDisplayOrder)
            (@arg credentials: -c --credentials <path> "Path to credentials.zip")
            (@arg name: -n --name <name> "The campaign name")
            (@arg groups: -g --groups <group> ... "Apply the campaign to the following groups")
            (@arg ("campaign-id"): -i --("campaign-id") [uuid] "(optional) Specify the campaign ID")
        )

        (@subcommand get =>
            (about: "Retrieve campaign information")
            (setting: AppSettings::ArgRequiredElseHelp)
            (setting: AppSettings::DeriveDisplayOrder)
            (@arg credentials: -c --credentials <path> "Path to credentials.zip")
            (@arg ("campaign-id"): -i --("campaign-id") <uuid> "The campaign ID")
        )

        (@subcommand launch =>
            (about: "Launch a created campaign")
            (setting: AppSettings::ArgRequiredElseHelp)
            (setting: AppSettings::DeriveDisplayOrder)
            (@arg credentials: -c --credentials <path> "Path to credentials.zip")
            (@arg ("campaign-id"): -i --("campaign-id") <uuid> "The campaign ID")
        )

        (@subcommand cancel =>
            (about: "Cancel a launched campaign")
            (setting: AppSettings::ArgRequiredElseHelp)
            (setting: AppSettings::DeriveDisplayOrder)
            (@arg credentials: -c --credentials <path> "Path to credentials.zip")
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
        builder.filter(Some("hyper"), LevelFilter::Info);
    }
    builder.init();
}
