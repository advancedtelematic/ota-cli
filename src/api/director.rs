use clap::ArgMatches;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs,
    path::Path,
    result,
    str::FromStr,
};
use uuid::Uuid;

use config::Config;
use error::{Error, Result};
use toml;
use util::print_resp;


/// Available director API methods.
pub trait DirectorApi {
    /// Create a new multi-target update.
    fn create_mtu(&mut Config, targets: &UpdateTargets) -> Result<Uuid>;
    /// Launch a previously created multi-target update.
    fn launch_mtu(&mut Config, device_id: Uuid, update_id: Uuid) -> Result<()>;
}


/// Make API calls to launch multi-target updates.
pub struct Director;

impl DirectorApi for Director {
    fn create_mtu(config: &mut Config, targets: &UpdateTargets) -> Result<Uuid> {
        let url = format!("{}api/v1/multi_target_updates", config.director);
        debug!("creating multi-target update: {:?}", targets);
        Ok(config
            .client()
            .post(&url)
            .json(targets)
            .headers(config.bearer_token()?)
            .send()?
            .json()?)
    }

    fn launch_mtu(config: &mut Config, device_id: Uuid, update_id: Uuid) -> Result<()> {
        let url = format!(
            "{}api/v1/admin/devices/{}/multi_target_update/{}",
            config.director, device_id, update_id
        );
        config
            .client()
            .put(&url)
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }
}


/// A TUF update target that can be applied to an ECU.
#[derive(Serialize, Deserialize, Debug)]
pub struct TufTarget {
    pub target: String,
    #[serde(rename = "targetLength")]
    pub length: u64,
    pub checksum: Checksum,
}

/// A request to update an ECU to a specific `TufTarget`.
#[derive(Serialize, Deserialize, Debug)]
pub struct TufUpdate {
    pub from: Option<TufTarget>,
    pub to: TufTarget,
    #[serde(rename = "targetFormat")]
    pub format: TargetFormat,
    #[serde(rename = "generateDiff")]
    pub generate_diff: bool,
}

/// A mapping from a hardware id's to TUF updates.
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateTargets {
    pub targets: HashMap<String, TufUpdate>,
}

impl UpdateTargets {
    /// Convert from `HardwareTargets` for creating a multi-target update.
    pub fn from(targets: HardwareTargets) -> Self {
        Self {
            targets: targets.targets.into_iter().map(Self::to_update).collect(),
        }
    }

    fn to_update(target: HardwareTarget) -> (String, TufUpdate) {
        let update = TufUpdate {
            from: None,
            to: TufTarget {
                target:   target.target,
                length:   target.length,
                checksum: Checksum {
                    method: target.method,
                    hash:   target.hash,
                },
            },
            format: target.format,
            generate_diff: false,
        };
        (target.hw_id, update)
    }
}


/// A target to apply to a specific type of hardware.
#[derive(Serialize, Deserialize)]
pub struct HardwareTarget {
    pub hw_id:  String,
    pub target: String,
    pub format: TargetFormat,
    pub length: u64,
    pub hash:   String,
    pub method: ChecksumMethod,
}

/// An input list of hardware update requests.
#[derive(Serialize, Deserialize)]
pub struct HardwareTargets {
    pub targets: Vec<HardwareTarget>,
}

impl HardwareTargets {
    /// Parse a toml file as a list of `HardwareTarget` items.
    pub fn from_file(input: impl AsRef<Path>) -> Result<Self> { Ok(toml::from_str(&fs::read_to_string(input).map_err(Error::Io)?)?) }
}


/// Available target types.
#[derive(Clone, Copy, Debug)]
pub enum TargetFormat {
    Binary,
    Ostree,
}

impl<'a> TargetFormat {
    /// Parse CLI arguments into a `TargetFormat`.
    pub fn from_matches(matches: &ArgMatches<'a>) -> Result<Self> {
        if matches.is_present("binary") {
            Ok(TargetFormat::Binary)
        } else if matches.is_present("ostree") {
            Ok(TargetFormat::Ostree)
        } else {
            Err(Error::Flag("Either --binary or --ostree flag is required".into()))
        }
    }
}

impl FromStr for TargetFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "binary" => Ok(TargetFormat::Binary),
            "ostree" => Ok(TargetFormat::Ostree),
            _ => Err(Error::Parse(format!("unknown `TargetFormat`: {}", s))),
        }
    }
}

impl Display for TargetFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let text = match self {
            TargetFormat::Binary => "BINARY",
            TargetFormat::Ostree => "OSTREE",
        };
        write!(f, "{}", text)
    }
}

impl Serialize for TargetFormat {
    fn serialize<S: Serializer>(&self, ser: S) -> result::Result<S::Ok, S::Error> { ser.serialize_str(&format!("{}", self)) }
}

impl<'de> Deserialize<'de> for TargetFormat {
    fn deserialize<D: Deserializer<'de>>(de: D) -> result::Result<Self, D::Error> {
        let s: String = Deserialize::deserialize(de)?;
        s.parse().map_err(|err| serde::de::Error::custom(format!("{}", err)))
    }
}


/// The checksum hash for a `TufTarget`.
#[derive(Serialize, Deserialize, Debug)]
pub struct Checksum {
    pub method: ChecksumMethod,
    pub hash:   String,
}

/// Available checksum methods for target metadata verification.
#[derive(Clone, Copy, Debug)]
pub enum ChecksumMethod {
    Sha256,
    Sha512,
}

impl FromStr for ChecksumMethod {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "sha256" => Ok(ChecksumMethod::Sha256),
            "sha512" => Ok(ChecksumMethod::Sha512),
            _ => Err(Error::Parse(format!("unknown `ChecksumMethod`: {}", s))),
        }
    }
}

impl Serialize for ChecksumMethod {
    fn serialize<S: Serializer>(&self, ser: S) -> result::Result<S::Ok, S::Error> {
        ser.serialize_str(match *self {
            ChecksumMethod::Sha256 => "sha256",
            ChecksumMethod::Sha512 => "sha512",
        })
    }
}

impl<'de> Deserialize<'de> for ChecksumMethod {
    fn deserialize<D: Deserializer<'de>>(de: D) -> result::Result<Self, D::Error> {
        let s: String = Deserialize::deserialize(de)?;
        s.parse().map_err(|err| serde::de::Error::custom(format!("{}", err)))
    }
}
