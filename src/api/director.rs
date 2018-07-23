use clap::ArgMatches;
use reqwest::{Client, Response};
use serde::{self, Deserialize, Deserializer};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs,
    path::Path,
    result,
    str::FromStr,
};
use toml;
use uuid::Uuid;

use config::Config;
use error::{Error, Result};
use http::{Http, HttpMethods};


/// Available director API methods.
pub trait DirectorApi {
    /// Create a new multi-target update.
    fn create_mtu(&mut Config, updates: &TufUpdates) -> Result<Response>;
    /// Launch a multi-target update for a device.
    fn launch_mtu(&mut Config, update: Uuid, device: Uuid) -> Result<Response>;
}


/// Make API calls to launch multi-target updates.
pub struct Director;

impl DirectorApi for Director {
    fn create_mtu(config: &mut Config, updates: &TufUpdates) -> Result<Response> {
        debug!("creating multi-target update: {:?}", updates);
        let req = Client::new()
            .post(&format!("{}api/v1/multi_target_updates", config.director))
            .json(updates)
            .build()?;
        Http::send(req, config.token()?)
    }

    fn launch_mtu(config: &mut Config, update: Uuid, device: Uuid) -> Result<Response> {
        debug!("launching multi-target update {} for device {}", update, device);
        Http::put(
            &format!("{}api/v1/admin/devices/{}/multi_target_update/{}", config.director, device, update),
            config.token()?,
        )
    }
}


/// An identifier for the type of hardware and applicable `Target`s.
type HardwareId = String;

/// Metadata describing an object that can be applied to an ECU.
#[derive(Serialize, Deserialize)]
pub struct TargetObject {
    pub name:    String,
    pub version: String,
    pub length:  Option<u64>,
    pub hash:    Option<String>,
    pub method:  Option<ChecksumMethod>,
}

/// A request to update some hardware type to a new `TargetObject`.
#[derive(Serialize, Deserialize)]
pub struct TargetRequest {
    pub target_format: Option<TargetFormat>,
    pub from:          Option<TargetObject>,
    pub to:            TargetObject,
    pub generate_diff: Option<bool>,
}

/// Parsed mapping from hardware identifiers to target requests.
#[derive(Serialize, Deserialize)]
pub struct TargetRequests {
    pub requests: HashMap<HardwareId, TargetRequest>,
}

impl TargetRequests {
    /// Parse a toml file into `TargetRequests`.
    pub fn from_file(input: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            requests: toml::from_str(&fs::read_to_string(input)?)?,
        })
    }
}


/// A request to update an ECU to a specific `TufTarget`.
#[derive(Serialize, Deserialize, Debug)]
pub struct TufUpdate {
    #[serde(rename = "targetFormat")]
    pub format: TargetFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<TufTarget>,
    pub to: TufTarget,
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
pub struct TufUpdates {
    pub targets: HashMap<HardwareId, TufUpdate>,
}

impl TufUpdates {
    /// Convert `TargetRequests` to `TufUpdates`.
    pub fn from(requests: TargetRequests) -> Result<Self> {
        Ok(Self {
            targets: requests
                .requests
                .into_iter()
                .map(|(id, req)| Ok((id, Self::to_update(req)?)))
                .collect::<Result<_>>()?,
        })
    }

    fn to_update(request: TargetRequest) -> Result<TufUpdate> {
        let format = request.target_format.unwrap_or(TargetFormat::Ostree);
        Ok(TufUpdate {
            format,
            generate_diff: request.generate_diff.unwrap_or(false),
            from: if let Some(from) = request.from {
                Some(Self::to_target(format, from)?)
            } else {
                None
            },
            to: Self::to_target(format, request.to)?,
        })
    }

    fn to_target(format: TargetFormat, target: TargetObject) -> Result<TufTarget> {
        let length = target.length.unwrap_or(0);
        if format == TargetFormat::Binary && length == 0 {
            Err(Error::Parse("binary target length cannot be 0".into()))?
        }
        Ok(TufTarget {
            target: format!("{}_{}", target.name, target.version),
            length,
            checksum: Checksum {
                method: target.method.unwrap_or(ChecksumMethod::Sha256),
                hash:   target.hash.unwrap_or(target.version),
            },
        })
    }
}


/// Available target types.
#[derive(Serialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TargetFormat {
    Binary,
    Ostree,
}

impl<'a> TargetFormat {
    /// Parse CLI arguments into a `TargetFormat`.
    pub fn from_args(args: &ArgMatches<'a>) -> Result<Self> {
        if args.is_present("binary") {
            Ok(TargetFormat::Binary)
        } else if args.is_present("ostree") {
            Ok(TargetFormat::Ostree)
        } else {
            Err(Error::Args("Either --binary or --ostree flag is required".into()))
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
#[derive(Serialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
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

impl<'de> Deserialize<'de> for ChecksumMethod {
    fn deserialize<D: Deserializer<'de>>(de: D) -> result::Result<Self, D::Error> {
        let s: String = Deserialize::deserialize(de)?;
        s.parse().map_err(|err| serde::de::Error::custom(format!("{}", err)))
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn parse_example_targets() {
        let requests = TargetRequests::from_file("examples/targets.toml").expect("parse toml");
        let updates = TufUpdates::from(requests).expect("targets").targets;
        assert_eq!(updates.len(), 2);

        if let Some(req) = updates.get("some-ecu-type") {
            assert_eq!(req.format, TargetFormat::Binary);
            assert_eq!(req.generate_diff, true);
            if let Some(ref from) = req.from {
                assert_eq!(from.target, "somefile_1.0.1");
            } else {
                panic!("missing `from` section")
            }
            assert_eq!(req.to.target, "somefile_1.0.2");
            assert_eq!(req.to.checksum.hash, "abcd012345678901234567890123456789012345678901234567890123456789");
            assert_eq!(req.to.checksum.method, ChecksumMethod::Sha256);
        } else {
            panic!("some-ecu-type not found");
        }

        if let Some(req) = updates.get("another ecu type") {
            assert_eq!(req.format, TargetFormat::Ostree);
            assert_eq!(req.generate_diff, false);
            assert!(req.from.is_none());
            assert_eq!(req.to.target, "my-branch_012345678901234567890123456789012345678901234567890123456789abcd");
            assert_eq!(req.to.checksum.hash, "012345678901234567890123456789012345678901234567890123456789abcd");
            assert_eq!(req.to.checksum.method, ChecksumMethod::Sha256);
        } else {
            panic!("another-ecu-type not found");
        }
    }
}
