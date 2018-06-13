use reqwest::{header::{Authorization, Bearer, Header},
              Client};
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashMap, str::FromStr};
use url::Url;
use uuid::Uuid;

use error::Error;
use token::Token;
use util::print_json;

/// A request to update multiple ECUs to new targets.
pub struct Director<'c> {
    client: &'c Client,
    server: Url,
    token: Token,
}

impl<'c> Director<'c> {
    pub fn new(client: &'c Client, server: Url, token: Token) -> Self {
        Director { client, server, token }
    }

    /// Create a new multi-target update in the director server.
    pub fn create_mtu(&self, targets: &UpdateTargets) -> Result<Uuid, Error> {
        debug!("creating multi-target update: {:?}", targets);
        let resp: Uuid = self.client
            .post(&format!("{}api/v1/multi_target_updates", self.server))
            .json(targets)
            .header(self.bearer()?)
            .send()?
            .json()?;
        Ok(resp)
    }

    /// Launch a previously created multi-target update.
    pub fn launch_mtu(&self, device_id: Uuid, update_id: Uuid) -> Result<(), Error> {
        let resp = self.client
            .put(&format!(
                "{}api/v1/admin/devices/{}/multi_target_update/{}",
                self.server, device_id, update_id
            ))
            .header(self.bearer()?)
            .send()?
            .json()?;
        print_json(resp)
    }

    fn bearer(&self) -> Result<impl Header, Error> {
        Ok(Authorization(Bearer {
            token: (self.token)()?.access_token,
        }))
    }
}

/// An update targetting a specific type of hardware.
#[derive(Serialize, Deserialize)]
pub struct Target {
    pub hw_id: String,
    pub target: String,
    pub length: u64,
    pub method: ChecksumMethod,
    pub hash: String,
}

/// A set of update targets for a specific `Device`.
#[derive(Deserialize)]
pub struct Targets {
    pub device: Device,
    pub targets: Vec<Target>,
}

/// An identifier for a specific device.
#[derive(Deserialize)]
pub struct Device {
    pub device_id: Uuid,
}

/// A mapping from hardware identifier's to a new `Update`.
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateTargets {
    pub targets: HashMap<String, Update>,
}

impl UpdateTargets {
    pub fn from(targets: &[Target], format: TargetFormat, generate_diff: bool) -> Self {
        UpdateTargets {
            targets: targets
                .into_iter()
                .map(|target| {
                    let update = Update {
                        from: None,
                        to: UpdateTarget {
                            target: target.target.clone(),
                            length: target.length,
                            checksum: Checksum {
                                method: target.method,
                                hash: target.hash.clone(),
                            },
                        },
                        format: format,
                        generate_diff: generate_diff,
                    };
                    (target.hw_id.clone(), update)
                })
                .collect::<HashMap<String, Update>>(),
        }
    }
}

/// Update an ECU to a specific `UpdateTarget`.
#[derive(Serialize, Deserialize, Debug)]
pub struct Update {
    pub from: Option<UpdateTarget>,
    pub to: UpdateTarget,
    #[serde(rename = "targetFormat")]
    pub format: TargetFormat,
    #[serde(rename = "generateDiff")]
    pub generate_diff: bool,
}

/// Available target types.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TargetFormat {
    Binary,
    Ostree,
}

/// The update target for a ECU.
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateTarget {
    pub target: String,
    #[serde(rename = "targetLength")]
    pub length: u64,
    pub checksum: Checksum,
}

/// The checksum hash for an `UpdateTarget`.
#[derive(Serialize, Deserialize, Debug)]
pub struct Checksum {
    pub method: ChecksumMethod,
    pub hash: String,
}

/// Available checksum methods for target metadata verification.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ChecksumMethod {
    Sha256,
    Sha512,
}

impl FromStr for ChecksumMethod {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "sha256" => Ok(ChecksumMethod::Sha256),
            "sha512" => Ok(ChecksumMethod::Sha512),
            _ => Err(Error::Checksum(s.into())),
        }
    }
}

impl Serialize for ChecksumMethod {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(match *self {
            ChecksumMethod::Sha256 => "sha256",
            ChecksumMethod::Sha512 => "sha512",
        })
    }
}

impl<'de> Deserialize<'de> for ChecksumMethod {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let s: String = Deserialize::deserialize(de)?;
        s.parse().map_err(|err| serde::de::Error::custom(format!("{}", err)))
    }
}
