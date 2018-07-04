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
    fn create_mtu(&mut Config, targets: &UpdateTargets) -> Result<()>;
    /// Launch a multi-target update for a device.
    fn launch_mtu(&mut Config, update: Uuid, device: Uuid) -> Result<()>;
}


/// Make API calls to launch multi-target updates.
pub struct Director;

impl DirectorApi for Director {
    fn create_mtu(config: &mut Config, targets: &UpdateTargets) -> Result<()> {
        debug!("creating multi-target update: {:?}", targets);
        config
            .client()
            .post(&format!("{}api/v1/multi_target_updates", config.director))
            .json(targets)
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn launch_mtu(config: &mut Config, update: Uuid, device: Uuid) -> Result<()> {
        debug!("launching multi-target update {} for device {}", update, device);
        let (upd, dev) = (update.hyphenated(), device.hyphenated());
        config
            .client()
            .put(&format!("{}api/v1/admin/devices/{}/multi_target_update/{}", config.director, dev, upd))
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }
}


/// An identifier for the type of hardware and applicable `Target`s.
type HardwareId = String;

/// A target referenced by `filepath` applicable to an ECU of type `hardware`.
#[derive(Serialize, Deserialize)]
pub struct Target {
    pub filepath: String,
    pub hardware: HardwareId,
    pub format:   TargetFormat,
    pub length:   u64,
    pub hash:     String,
    pub method:   ChecksumMethod,
}

/// Target requests to update hardware.
#[derive(Serialize, Deserialize)]
pub struct Targets {
    pub targets: Vec<Target>,
}

impl Targets {
    /// Parse a toml file into `Target` update requests.
    pub fn from_file(input: impl AsRef<Path>) -> Result<Self> { Ok(toml::from_str(&fs::read_to_string(input).map_err(Error::Io)?)?) }
}


/// A request to update an ECU to a specific `TufTarget`.
#[derive(Serialize, Deserialize, Debug)]
pub struct TufUpdate {
    /// Optionally confirm that the current target matches `from` before updating.
    pub from: Option<TufTarget>,
    pub to: TufTarget,
    #[serde(rename = "targetFormat")]
    pub format: TargetFormat,
    #[serde(rename = "generateDiff")]
    pub generate_diff: bool,
}

/// A TUF target for an ECU.
#[derive(Serialize, Deserialize, Debug)]
pub struct TufTarget {
    pub target: String,
    #[serde(rename = "targetLength")]
    pub length: u64,
    pub checksum: Checksum,
}

/// An update request for each `EcuSerial` to a `TufUpdate` target.
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateTargets {
    pub targets: HashMap<HardwareId, TufUpdate>,
}

impl UpdateTargets {
    /// Convert from `Targets` for creating a multi-target update.
    pub fn from(targets: Targets) -> Self {
        Self {
            targets: targets.targets.into_iter().map(Self::to_update).collect(),
        }
    }

    fn to_update(target: Target) -> (String, TufUpdate) {
        let update = TufUpdate {
            from: None,
            to: TufTarget {
                target:   target.filepath,
                length:   target.length,
                checksum: Checksum {
                    method: target.method,
                    hash:   target.hash,
                },
            },
            format: target.format,
            generate_diff: false,
        };
        (target.hardware, update)
    }
}


/// Available target types.
#[derive(Clone, Copy, Debug)]
pub enum TargetFormat {
    Binary,
    Ostree,
}

impl<'a> TargetFormat {
    /// Parse CLI arguments into a `TargetFormat`.
    pub fn from_flags(flags: &ArgMatches<'a>) -> Result<Self> {
        if flags.is_present("binary") {
            Ok(TargetFormat::Binary)
        } else if flags.is_present("ostree") {
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
