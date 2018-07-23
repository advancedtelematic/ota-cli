#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate log;
extern crate ota;

use clap::{AppSettings, ArgMatches};
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

use ota::{
    command::{Command, Exec},
    error::Result,
    http::Http,
};

fn main() -> Result<()> {
    let args = parse_args();
    Builder::from_default_env()
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .parse(args.value_of("level").unwrap_or("INFO"))
        .filter(Some("tokio"), LevelFilter::Info)
        .init();

    let (cmd, args) = args.subcommand();
    let cmd = cmd.parse::<Command>()?;
    let args = args.expect("cli args");
    cmd.exec(args, Http::print_response)
}

fn parse_args<'a>() -> ArgMatches<'a> {
    clap_app!((crate_name!()) =>
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

        (@subcommand list =>
          (about: "List campaign information")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg all: -a --all conflicts_with[campaign stats] "List all campaigns")
          (@arg campaign: -c --campaign [uuid] conflicts_with[all] "The campaign id")
          (@arg stats: -s --stats conflicts_with[all] "List campaign stats")
        )

        (@subcommand create =>
          (about: "Create a new campaign")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg update: -u --update <uuid> "Multi-target update id")
          (@arg name: -n --name <name> "A campaign name")
          (@arg groups: -g --groups <uuid> ... "Apply the campaign to these groups")
        )

        (@subcommand launch =>
          (about: "Launch a created campaign")
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

        (@subcommand list =>
          (about: "List devices")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg all: -a --all conflicts_with[device] "List all devices")
          (@arg device: -d --device [uuid] conflicts_with[group all] "List information about this device")
          (@arg group: -g --group [uuid] conflicts_with[device all] "List the devices in this group")
        )

       /*
        (@subcommand create =>
          (about: "Create a new device")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg name: -n --name <name> "A device display name")
          (@arg id: -i --id <id> "A device identifier (e.g. VIN)")
          (@arg vehicle: -v --vehicle conflicts_with[other] "Vehicle device type")
          (@arg other: -o --other conflicts_with[vehicle] "Other device type")
        )
       */

        (@subcommand delete =>
          (about: "Delete an existing device")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg device: -d --device <uuid> "The device id")
        )
      )

      (@subcommand group =>
        (about: "Manage device groups")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (setting: AppSettings::DeriveDisplayOrder)
        (setting: AppSettings::InferSubcommands)
        (setting: AppSettings::UnifiedHelpMessage)

        (@subcommand list =>
          (about: "List groups and their devices")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg all: -a --all conflicts_with[group device] "List all groups")
          (@arg group: -g --group [uuid] conflicts_with[device all] "List the devices in this group")
          (@arg device: -d --device [uuid] conflicts_with[group all] "List the groups for this device")
        )

        (@subcommand create =>
          (about: "Create a new group")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg name: -n --name <name> "The group name")
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

        (@subcommand rename =>
          (about: "Rename an existing group")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg group: -g --group <uuid> "The group to rename")
          (@arg name: -n --name <name> "The new group name")
        )
      )

      (@subcommand package =>
        (about: "Manage OTA packages")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (setting: AppSettings::DeriveDisplayOrder)
        (setting: AppSettings::InferSubcommands)
        (setting: AppSettings::UnifiedHelpMessage)

       /*
        (@subcommand list =>
          (about: "List available packages")
        )
       */

        (@subcommand add =>
          (about: "Add a single package")
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

        (@subcommand fetch =>
          (about: "Fetch package contents")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg name: -n --name <name> "The package name")
          (@arg version: -v --version <version> "The package version")
        )

        (@subcommand upload =>
          (about: "Upload multiple packages")
          (setting: AppSettings::ArgRequiredElseHelp)
          (setting: AppSettings::DeriveDisplayOrder)
          (setting: AppSettings::UnifiedHelpMessage)
          (@arg packages: -p --packages <toml> "Package metadata file")
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
