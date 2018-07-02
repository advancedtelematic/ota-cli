use std::str::FromStr;

use error::{Error, Result};


/// Available command-line interface sub-commands.
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Command {
    Init,
    Campaign,
    Group,
    Package,
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "init" => Ok(Command::Init),
            "campaign" => Ok(Command::Campaign),
            "group" => Ok(Command::Group),
            "package" => Ok(Command::Package),
            _ => Err(Error::Command(format!("unknown command: {}", s))),
        }
    }
}

/// Available campaign sub-commands.
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Campaign {
    Create,
    Launch,
    Get,
    Stats,
    Cancel,
}

impl FromStr for Campaign {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "create" => Ok(Campaign::Create),
            "launch" => Ok(Campaign::Launch),
            "get" => Ok(Campaign::Get),
            "stats" => Ok(Campaign::Stats),
            "cancel" => Ok(Campaign::Cancel),
            _ => Err(Error::Command(format!("unknown campaign subcommand: {}", s))),
        }
    }
}

/// Available group sub-commands.
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Group {
    Create,
    Rename,
    List,
    Add,
    Remove,
}

impl FromStr for Group {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "create" => Ok(Group::Create),
            "rename" => Ok(Group::Rename),
            "list" => Ok(Group::List),
            "add" => Ok(Group::Add),
            "remove" => Ok(Group::Remove),
            _ => Err(Error::Command(format!("unknown group subcommand: {}", s))),
        }
    }
}

/// Available package sub-commands.
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Package {
    Add,
}

impl FromStr for Package {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "add" => Ok(Package::Add),
            _ => Err(Error::Command(format!("unknown package subcommand: {}", s))),
        }
    }
}
