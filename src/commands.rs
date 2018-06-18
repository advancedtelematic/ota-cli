use std::str::FromStr;

use error::{Error, Result};

/// Available command-line interface sub-commands.
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Command {
    Campaign,
    Package,
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "campaign" => Ok(Command::Campaign),
            "package" => Ok(Command::Package),
            _ => Err(Error::Command(s.into())),
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
            _ => Err(Error::CommandCampaign(s.into())),
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
            _ => Err(Error::CommandPackage(s.into())),
        }
    }
}
