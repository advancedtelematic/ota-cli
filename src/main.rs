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
extern crate urlencoding;
extern crate uuid;
extern crate zip;

mod api;
mod commands;
mod config;
mod error;
mod util;

use clap::{AppSettings, ArgMatches};
use uuid::Uuid;

use api::{
    campaigner::{Campaigner, CampaignerApi},
    registry::{Registry, RegistryApi},
    reposerver::{Reposerver, ReposerverApi, TargetPackage},
};
use commands::{Campaign, Command, Group, Package};
use config::Config;
use error::Error;
use util::start_logging;


fn main() -> Result<(), Error> {
    let args = parse_args();
    let (command, flags) = args.subcommand();
    let flags = flags.expect("sub-command flags");
    start_logging(args.value_of("level").unwrap_or("INFO"));

    match command.parse()? {
        Command::Init => Config::init_matches(&flags),

        Command::Campaign => {
            let mut config = Config::load_default()?;
            let (campaign, flags) = flags.subcommand();
            let flags = flags.expect("campaign flags");
            let id = || flags.value_of("id").expect("--id").parse::<Uuid>();

            match campaign.parse()? {
                Campaign::Create => Campaigner::create_from_matches(&mut config, &flags),
                Campaign::Get => Campaigner::get(&mut config, id()?),
                Campaign::Launch => Campaigner::launch(&mut config, id()?),
                Campaign::Stats => Campaigner::stats(&mut config, id()?),
                Campaign::Cancel => Campaigner::cancel(&mut config, id()?),
            }
        }

        Command::Group => {
            let mut config = Config::load_default()?;
            let (group, flags) = flags.subcommand();
            let flags = flags.expect("group flags");
            let id = || flags.value_of("group").expect("--group").parse::<Uuid>();
            let name = || flags.value_of("name").expect("--name");
            let device = || flags.value_of("device").expect("--device").parse::<Uuid>();

            match group.parse()? {
                Group::Create => Registry::create_group(&mut config, name()),
                Group::Rename => Registry::rename_group(&mut config, id()?, name()),
                Group::List => Registry::delegate(&mut config, flags),
                Group::Add => Registry::add_to_group(&mut config, id()?, device()?),
                Group::Remove => Registry::remove_from_group(&mut config, id()?, device()?),
            }
        }

        Command::Package => {
            let mut config = Config::load_default()?;
            let (package, flags) = flags.subcommand();
            let flags = flags.expect("package flags");

            match package.parse()? {
                Package::Add => Reposerver::add_package(&mut config, TargetPackage::from_matches(flags)?),
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

      (@arg level: -l --level [level] "(optional) Set the logging level")

      (@subcommand init =>
        (about: "Set config values before starting")
        (setting: AppSettings::ArgRequiredElseHelp)
        (setting: AppSettings::DeriveDisplayOrder)
        (@arg credentials: -z --credentials <zip> "Path to credentials.zip")
        (@arg campaigner: -c --campaigner <url> "Campaigner URL")
        (@arg director: -d --director <url> "Director URL")
        (@arg registry: -r --registry <url> "Device Registry URL")
      )

      (@subcommand campaign =>
        (about: "Manage OTA campaigns")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (setting: AppSettings::DeriveDisplayOrder)

        (@subcommand create =>
          (about: "Create a new campaign")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (@arg name: -n --name <name> "The campaign name")
          (@arg groups: -g --groups <uuid> ... "Apply the campaign to the following groups")
          (@arg targets: -t --targets <toml> "Config file for the update targets")
        )

        (@subcommand get =>
          (about: "Retrieve campaign information")
          (setting: AppSettings::ArgRequiredElseHelp)
          (@arg id: -i --id <uuid> "The campaign ID")
        )

        (@subcommand launch =>
          (about: "Launch a created campaign")
          (setting: AppSettings::ArgRequiredElseHelp)
          (@arg id: -i --id <uuid> "The campaign ID")
        )

        (@subcommand stats =>
          (about: "Retrieve stats from a campaign")
          (setting: AppSettings::ArgRequiredElseHelp)
          (@arg id: -i --id <uuid> "The campaign ID")
        )

        (@subcommand cancel =>
          (about: "Cancel a launched campaign")
          (setting: AppSettings::ArgRequiredElseHelp)
          (@arg id: -i --id <uuid> "The campaign ID")
        )
      )

      (@subcommand group =>
        (about: "Manage OTA groups")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (setting: AppSettings::DeriveDisplayOrder)

        (@subcommand create =>
          (about: "Create a new group")
          (setting: AppSettings::ArgRequiredElseHelp)
          (@arg name: -n --name <name> "The group name")
        )

        (@subcommand rename =>
          (about: "Rename an existing group")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (@arg group: -i --id <uuid> "The group ID")
          (@arg name: -n --name <name> "The new group name")
        )

        (@subcommand list =>
          (about: "List groups and their devices")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (@arg all: -a --all conflicts_with[group device] "List all groups")
          (@arg group: -g --group [uuid] conflicts_with[device all] "List the devices in this group")
          (@arg device: -d --device [uuid] conflicts_with[group all] "List the groups for this device")
        )

        (@subcommand add =>
          (about: "Add a device to a group")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (@arg group: -g --group <uuid> "The group to add the device to")
          (@arg device: -d --device <uuid> "The device to add")
        )

        (@subcommand remove =>
          (about: "Remove a device from a group")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (@arg group: -g --group <uuid> "The group to remove the device from")
          (@arg device: -d --device <uuid> "The device to remove")
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
          (@arg hardware: -h --hardware <id> ... "Package works on these hardware IDs")
          (@arg path: -p --path [path] conflicts_with[url] "Path to package contents")
          (@arg url: -u --url [url] conflicts_with[path] "URL to package contents")
          (@arg binary: -b --binary conflicts_with[ostree] "Binary package format")
          (@arg ostree: -o --ostree conflicts_with[binary] "OSTree package format")
        )
      )
    ).get_matches()
}
