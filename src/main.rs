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

use api::{
    campaigner::{Campaigner, CampaignerApi},
    director::{Director, DirectorApi, Targets, UpdateTargets},
    registry::{DeviceType, Registry, RegistryApi},
    reposerver::{Reposerver, ReposerverApi, TargetPackage},
};
use commands::{Campaign, Command, Device, Group, Package, Update};
use config::Config;
use error::Result;
use util::start_logging;

fn main() -> Result<()> {
    let args = parse_args();
    let (command, flags) = args.subcommand();
    let flags = flags.expect("sub-command flags");
    start_logging(args.value_of("level").unwrap_or("INFO"));

    match command.parse()? {
        Command::Init => Config::init_flags(&flags),

        Command::Campaign => {
            let mut config = Config::load_default()?;
            let (campaign_cmd, flags) = flags.subcommand();
            let flags = flags.expect("campaign flags");
            let campaign = || flags.value_of("campaign").expect("--campaign").parse();

            match campaign_cmd.parse()? {
                Campaign::Create => Campaigner::create_from_flags(&mut config, &flags),
                Campaign::Get => Campaigner::get(&mut config, campaign()?),
                Campaign::Launch => Campaigner::launch(&mut config, campaign()?),
                Campaign::Stats => Campaigner::stats(&mut config, campaign()?),
                Campaign::Cancel => Campaigner::cancel(&mut config, campaign()?),
            }
        }

        Command::Device => {
            let mut config = Config::load_default()?;
            let (device_cmd, flags) = flags.subcommand();
            let flags = flags.expect("device flags");
            let name = || flags.value_of("name").expect("--name");
            let id = || flags.value_of("id").expect("--id");

            match device_cmd.parse()? {
                Device::Create => Registry::create_device(&mut config, name(), id(), DeviceType::from_flags(flags)?),
                Device::List => Registry::list_device_flags(&mut config, flags),
            }
        }

        Command::Group => {
            let mut config = Config::load_default()?;
            let (group_cmd, flags) = flags.subcommand();
            let flags = flags.expect("group flags");
            let group = || flags.value_of("group").expect("--group").parse();
            let device = || flags.value_of("device").expect("--device").parse();
            let name = || flags.value_of("name").expect("--name");

            match group_cmd.parse()? {
                Group::Create => Registry::create_group(&mut config, name()),
                Group::Rename => Registry::rename_group(&mut config, group()?, name()),
                Group::List => Registry::list_group_flags(&mut config, flags),
                Group::Add => Registry::add_to_group(&mut config, group()?, device()?),
                Group::Remove => Registry::remove_from_group(&mut config, group()?, device()?),
            }
        }

        Command::Package => {
            let mut config = Config::load_default()?;
            let (package_cmd, flags) = flags.subcommand();
            let flags = flags.expect("package flags");
            let name = || flags.value_of("name").expect("--name");
            let version = || flags.value_of("version").expect("--version");

            match package_cmd.parse()? {
                Package::Add => Reposerver::add_package(&mut config, TargetPackage::from_flags(flags)?),
                Package::Get => Reposerver::get_package(&mut config, name(), version()),
            }
        }

        Command::Update => {
            let mut config = Config::load_default()?;
            let (update_cmd, flags) = flags.subcommand();
            let flags = flags.expect("update flags");
            let update = || flags.value_of("update").expect("--update").parse();
            let device = || flags.value_of("device").expect("--device").parse();
            let targets = || flags.value_of("targets").expect("--targets");

            match update_cmd.parse()? {
                Update::Create => Director::create_mtu(&mut config, &UpdateTargets::from(Targets::from_file(targets())?)),
                Update::Launch => Director::launch_mtu(&mut config, update()?, device()?)
            }
        }
    }
}

fn parse_args<'a>() -> ArgMatches<'a> {
    clap_app!(("ota-cli") =>
      (version: crate_version!())
      (setting: AppSettings::SubcommandRequiredElseHelp)
      (setting: AppSettings::DeriveDisplayOrder)
      (setting: AppSettings::InferSubcommands)
      (setting: AppSettings::VersionlessSubcommands)
      (setting: AppSettings::UnifiedHelpMessage)

      (@arg level: -l --level [level] +global "Set the logging level")

      (@subcommand init =>
        (about: "Set config values before starting")
        (setting: AppSettings::ArgRequiredElseHelp)
        (setting: AppSettings::DeriveDisplayOrder)
        (setting: AppSettings::UnifiedHelpMessage)
        (@arg credentials: -z --credentials <zip> "Path to credentials.zip")
        (@arg campaigner: -c --campaigner <url> "Campaigner URL")
        (@arg director: -d --director <url> "Director URL")
        (@arg registry: -r --registry <url> "Device Registry URL")
      )

      (@subcommand campaign =>
        (about: "Manage OTA campaigns")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (setting: AppSettings::DeriveDisplayOrder)
        (setting: AppSettings::InferSubcommands)
        (setting: AppSettings::UnifiedHelpMessage)

        (@subcommand create =>
          (about: "Create a new campaign")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg update: -u --update <uuid> "Multi-target update id")
          (@arg name: -n --name <name> "A campaign name")
          (@arg groups: -g --groups <uuid> ... "Apply the campaign to these groups")
        )

        (@subcommand get =>
          (about: "Retrieve campaign information")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg campaign: -c --campaign <uuid> "The campaign id")
        )

        (@subcommand launch =>
          (about: "Launch a created campaign")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg campaign: -c --campaign <uuid> "The campaign id")
        )

        (@subcommand stats =>
          (about: "Retrieve stats from a campaign")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg campaign: -c --campaign <uuid> "The campaign id")
        )

        (@subcommand cancel =>
          (about: "Cancel a launched campaign")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg campaign: -c --campaign <uuid> "The campaign id")
        )
      )

      (@subcommand device =>
        (about: "Manage OTA devices")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (setting: AppSettings::DeriveDisplayOrder)
        (setting: AppSettings::InferSubcommands)
        (setting: AppSettings::UnifiedHelpMessage)

        (@subcommand create =>
          (about: "Create a new device")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg name: -n --name <name> "A device name")
          (@arg id: -i --id <id> "A device identifier")
          (@arg vehicle: -v --vehicle conflicts_with[other] "Vehicle device type")
          (@arg other: -o --other conflicts_with[vehicle] "Other device type")
        )

        (@subcommand list =>
          (about: "List devices")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg all: -a --all conflicts_with[device] "List all devices")
          (@arg device: -d --device [uuid] conflicts_with[group all] "List information about this device")
          (@arg group: -g --group [uuid] conflicts_with[device all] "List the devices in this group")
        )
      )

      (@subcommand group =>
        (about: "Manage device groups")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (setting: AppSettings::DeriveDisplayOrder)
        (setting: AppSettings::InferSubcommands)
        (setting: AppSettings::UnifiedHelpMessage)

        (@subcommand create =>
          (about: "Create a new group")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg name: -n --name <name> "The group name")
        )

        (@subcommand rename =>
          (about: "Rename an existing group")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg group: -g --group <uuid> "The group to rename")
          (@arg name: -n --name <name> "The new group name")
        )

        (@subcommand list =>
          (about: "List groups and their devices")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg all: -a --all conflicts_with[group device] "List all groups")
          (@arg group: -g --group [uuid] conflicts_with[device all] "List the devices in this group")
          (@arg device: -d --device [uuid] conflicts_with[group all] "List the groups for this device")
        )

        (@subcommand add =>
          (about: "Add a device to a group")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg group: -g --group <uuid> "The group to add the device to")
          (@arg device: -d --device <uuid> "The device to add")
        )

        (@subcommand remove =>
          (about: "Remove a device from a group")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg group: -g --group <uuid> "The group to remove the device from")
          (@arg device: -d --device <uuid> "The device to remove")
        )
      )

      (@subcommand package =>
        (about: "Manage OTA packages")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (setting: AppSettings::DeriveDisplayOrder)
        (setting: AppSettings::InferSubcommands)
        (setting: AppSettings::UnifiedHelpMessage)

        (@subcommand add =>
          (about: "Add a new package")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg name: -n --name <name> "The package name")
          (@arg version: -v --version <version> "The package version")
          (@arg hardware: -h --hardware <id> ... "Package works on these hardware IDs")
          (@arg path: -p --path [path] conflicts_with[url] "Path to package contents")
          (@arg url: -u --url [url] conflicts_with[path] "URL to package contents")
          (@arg binary: -b --binary conflicts_with[ostree] "Binary package format")
          (@arg ostree: -o --ostree conflicts_with[binary] "OSTree package format")
        )

        (@subcommand get =>
          (about: "Get package contents")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg name: -n --name <name> "The package name")
          (@arg version: -v --version <version> "The package version")
        )
      )

      (@subcommand update =>
        (about: "Manage multi-target updates")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (setting: AppSettings::DeriveDisplayOrder)
        (setting: AppSettings::InferSubcommands)
        (setting: AppSettings::UnifiedHelpMessage)
 
        (@subcommand create =>
          (about: "Create a multi-target update")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg targets: -t --targets <toml> "Update targets file")
        )

        (@subcommand launch =>
          (about: "Launch a multi-target update")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg update: -u --update <uuid> "Multi-target update id")
          (@arg device: -d --device <uuid> "Apply to this device")
        )
      )
    ).get_matches()
}
