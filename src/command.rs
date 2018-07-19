use clap::ArgMatches;
use reqwest::Response;
use std::str::FromStr;

use api::{
    campaigner::{Campaigner, CampaignerApi},
    director::{Director, DirectorApi, TargetRequests, TufUpdates},
    registry::{DeviceType, Registry, RegistryApi},
    reposerver::{Reposerver, ReposerverApi, TargetPackages, TufPackage, TufPackages},
};
use config::Config;
use error::{Error, Result};


/// Execute an HTTP request.
pub trait HttpRequest<'a> {
    fn exec(&self, flags: &ArgMatches<'a>) -> Result<Response>;
}


/// Available CLI sub-commands.
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Command {
    Init,
    Campaign,
    Device,
    Group,
    Package,
    Update,
}

impl<'a> HttpRequest<'a> for Command {
    fn exec(&self, flags: &ArgMatches<'a>) -> Result<Response> {
        let (cmd, args) = flags.subcommand();
        let args = args.expect("sub-command args");
        #[cfg_attr(rustfmt, rustfmt_skip)]
        match self {
            Command::Init     => panic!("Command::init does not handle HTTP requests"),
            Command::Campaign => cmd.parse::<Campaign>()?.exec(args),
            Command::Device   => cmd.parse::<Device>()?.exec(args),
            Command::Group    => cmd.parse::<Group>()?.exec(args),
            Command::Package  => cmd.parse::<Package>()?.exec(args),
            Command::Update   => cmd.parse::<Update>()?.exec(args),
        }
    }
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        match s.to_lowercase().as_ref() {
            "init"     => Ok(Command::Init),
            "campaign" => Ok(Command::Campaign),
            "device"   => Ok(Command::Device),
            "group"    => Ok(Command::Group),
            "package"  => Ok(Command::Package),
            "update"   => Ok(Command::Update),
            _ => Err(Error::Command(format!("unknown command: {}", s))),
        }
    }
}


/// Available campaign sub-commands.
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Campaign {
    List,
    Create,
    Launch,
    Cancel,
}

impl<'a> HttpRequest<'a> for Campaign {
    fn exec(&self, flags: &ArgMatches<'a>) -> Result<Response> {
        let mut config = Config::load_default()?;
        let campaign = || flags.value_of("campaign").expect("--campaign").parse();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        match self {
            Campaign::List   => Campaigner::list_from_flags(&mut config, flags),
            Campaign::Create => Campaigner::create_from_flags(&mut config, flags),
            Campaign::Launch => Campaigner::launch_campaign(&mut config, campaign()?),
            Campaign::Cancel => Campaigner::cancel_campaign(&mut config, campaign()?),
        }
    }
}

impl FromStr for Campaign {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        match s.to_lowercase().as_ref() {
            "list"   => Ok(Campaign::List),
            "create" => Ok(Campaign::Create),
            "launch" => Ok(Campaign::Launch),
            "cancel" => Ok(Campaign::Cancel),
            _ => Err(Error::Command(format!("unknown campaign subcommand: {}", s))),
        }
    }
}


/// Available device sub-commands.
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Device {
    List,
    Create,
    Delete,
}

impl<'a> HttpRequest<'a> for Device {
    fn exec(&self, flags: &ArgMatches<'a>) -> Result<Response> {
        let mut config = Config::load_default()?;
        let device = || flags.value_of("device").expect("--device").parse();
        let name = || flags.value_of("name").expect("--name");
        let id = || flags.value_of("id").expect("--id");

        #[cfg_attr(rustfmt, rustfmt_skip)]
        match self {
            Device::List   => Registry::list_device_flags(&mut config, flags),
            Device::Create => Registry::create_device(&mut config, name(), id(), DeviceType::from_flags(flags)?),
            Device::Delete => Registry::delete_device(&mut config, device()?),
        }
    }
}

impl FromStr for Device {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        match s.to_lowercase().as_ref() {
            "list"   => Ok(Device::List),
            "create" => Ok(Device::Create),
            "delete" => Ok(Device::Delete),
            _ => Err(Error::Command(format!("unknown device subcommand: {}", s))),
        }
    }
}


/// Available group sub-commands.
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Group {
    List,
    Create,
    Add,
    Rename,
    Remove,
}

impl<'a> HttpRequest<'a> for Group {
    fn exec(&self, flags: &ArgMatches<'a>) -> Result<Response> {
        let mut config = Config::load_default()?;
        let group = || flags.value_of("group").expect("--group").parse();
        let device = || flags.value_of("device").expect("--device").parse();
        let name = || flags.value_of("name").expect("--name");

        #[cfg_attr(rustfmt, rustfmt_skip)]
        match self {
            Group::List   => Registry::list_group_flags(&mut config, flags),
            Group::Create => Registry::create_group(&mut config, name()),
            Group::Add    => Registry::add_to_group(&mut config, group()?, device()?),
            Group::Remove => Registry::remove_from_group(&mut config, group()?, device()?),
            Group::Rename => Registry::rename_group(&mut config, group()?, name()),
        }
    }
}

impl FromStr for Group {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        match s.to_lowercase().as_ref() {
            "list"   => Ok(Group::List),
            "create" => Ok(Group::Create),
            "add"    => Ok(Group::Add),
            "rename" => Ok(Group::Rename),
            "remove" => Ok(Group::Remove),
            _ => Err(Error::Command(format!("unknown group subcommand: {}", s))),
        }
    }
}


/// Available package sub-commands.
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Package {
    List,
    Add,
    Fetch,
    Upload,
}

impl<'a> HttpRequest<'a> for Package {
    fn exec(&self, flags: &ArgMatches<'a>) -> Result<Response> {
        let mut config = Config::load_default()?;
        let name = || flags.value_of("name").expect("--name");
        let version = || flags.value_of("version").expect("--version");
        let packages = || flags.value_of("packages").expect("--packages");

        #[cfg_attr(rustfmt, rustfmt_skip)]
        match self {
            Package::List   => panic!("API not yet supported"),
            Package::Add    => Reposerver::add_package(&mut config, TufPackage::from_flags(flags)?),
            Package::Fetch  => Reposerver::get_package(&mut config, name(), version()),
            Package::Upload => Reposerver::add_packages(&mut config, TufPackages::from(TargetPackages::from_file(packages())?)?),
        }
    }
}

impl FromStr for Package {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        match s.to_lowercase().as_ref() {
            "list"   => Ok(Package::List),
            "add"    => Ok(Package::Add),
            "fetch"  => Ok(Package::Fetch),
            "upload" => Ok(Package::Upload),
            _ => Err(Error::Command(format!("unknown package subcommand: {}", s))),
        }
    }
}


/// Available update sub-commands.
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Update {
    Create,
    Launch,
}

impl<'a> HttpRequest<'a> for Update {
    fn exec(&self, flags: &ArgMatches<'a>) -> Result<Response> {
        let mut config = Config::load_default()?;
        let update = || flags.value_of("update").expect("--update").parse();
        let device = || flags.value_of("device").expect("--device").parse();
        let targets = || flags.value_of("targets").expect("--targets");

        match self {
            Update::Create => Director::create_mtu(&mut config, &TufUpdates::from(TargetRequests::from_file(targets())?)?),
            Update::Launch => Director::launch_mtu(&mut config, update()?, device()?),
        }
    }
}

impl FromStr for Update {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        match s.to_lowercase().as_ref() {
            "create" => Ok(Update::Create),
            "launch" => Ok(Update::Launch),
            _ => Err(Error::Command(format!("unknown update subcommand: {}", s))),
        }
    }
}
