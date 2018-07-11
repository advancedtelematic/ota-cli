use std::str::FromStr;

use error::{Error, Result};


/// Available command-line interface sub-commands.
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Command {
    Init,
    Campaign,
    Device,
    Group,
    Package,
    Update,
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "init" => Ok(Command::Init),
            "campaign" => Ok(Command::Campaign),
            "device" => Ok(Command::Device),
            "group" => Ok(Command::Group),
            "package" => Ok(Command::Package),
            "update" => Ok(Command::Update),
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

impl FromStr for Campaign {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "list" => Ok(Campaign::List),
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

impl FromStr for Device {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "list" => Ok(Device::List),
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

impl FromStr for Group {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "list" => Ok(Group::List),
            "create" => Ok(Group::Create),
            "add" => Ok(Group::Add),
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
}

impl FromStr for Package {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "list" => Ok(Package::List),
            "add" => Ok(Package::Add),
            "fetch" => Ok(Package::Fetch),
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

impl FromStr for Update {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "create" => Ok(Update::Create),
            "launch" => Ok(Update::Launch),
            _ => Err(Error::Command(format!("unknown update subcommand: {}", s))),
        }
    }
}
