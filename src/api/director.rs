use reqwest::{header::{Authorization, Bearer, Headers},
              Client};
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::{self,
          collections::HashMap,
          fmt::{self, Display, Formatter},
          str::FromStr};
use url::Url;
use uuid::Uuid;

use api::auth_plus::Token;
use error::{Error, Result};
use util::print_resp;

/// Available director API methods.
pub trait Director {
    /// Create a new multi-target update.
    fn create_mtu(&self, targets: &UpdateTargets) -> Result<Uuid>;
    /// Launch a previously created multi-target update.
    fn launch_mtu(&self, device_id: Uuid, update_id: Uuid) -> Result<()>;
}

/// Handle API calls to director to launch multi-target updates.
pub struct DirectorHandler<'c> {
    client: &'c Client,
    server: Url,
    token: Token,
}

impl<'c> DirectorHandler<'c> {
    pub fn new(client: &'c Client, server: Url, token: Token) -> Self {
        DirectorHandler { client, server, token }
    }

    fn auth(&self) -> Result<Headers> {
        let mut headers = Headers::new();
        if let Some(resp) = (self.token)()? {
            headers.set(Authorization(Bearer {
                token: resp.access_token,
            }));
        }
        Ok(headers)
    }
}

impl<'c> Director for DirectorHandler<'c> {
    fn create_mtu(&self, targets: &UpdateTargets) -> Result<Uuid> {
        debug!("creating multi-target update: {:?}", targets);
        Ok(self.client
            .post(&format!("{}api/v1/multi_target_updates", self.server))
            .json(targets)
            .headers(self.auth()?)
            .send()?
            .json()?)
    }

    fn launch_mtu(&self, device_id: Uuid, update_id: Uuid) -> Result<()> {
        self.client
            .put(&format!(
                "{}api/v1/admin/devices/{}/multi_target_update/{}",
                self.server, device_id, update_id
            ))
            .headers(self.auth()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
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
    /// Convert a list of `Target`s to `UpdateTargets`.
    pub fn from(targets: &[Target], format: TargetFormat, generate_diff: bool) -> Self {
        let to_update = |target: &Target| Update {
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

        UpdateTargets {
            targets: targets
                .into_iter()
                .map(|target| (target.hw_id.clone(), to_update(&target)))
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

impl FromStr for TargetFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_ref() {
            "BINARY" => Ok(TargetFormat::Binary),
            "OSTREE" => Ok(TargetFormat::Ostree),
            _ => Err(Error::TargetFormat(s.into())),
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

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "sha256" => Ok(ChecksumMethod::Sha256),
            "sha512" => Ok(ChecksumMethod::Sha512),
            _ => Err(Error::Checksum(s.into())),
        }
    }
}

impl Serialize for ChecksumMethod {
    fn serialize<S: Serializer>(&self, ser: S) -> std::result::Result<S::Ok, S::Error> {
        ser.serialize_str(match *self {
            ChecksumMethod::Sha256 => "sha256",
            ChecksumMethod::Sha512 => "sha512",
        })
    }
}

impl<'de> Deserialize<'de> for ChecksumMethod {
    fn deserialize<D: Deserializer<'de>>(de: D) -> std::result::Result<Self, D::Error> {
        let s: String = Deserialize::deserialize(de)?;
        s.parse().map_err(|err| serde::de::Error::custom(format!("{}", err)))
    }
}
