use clap::ArgMatches;
use reqwest::Client;
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};
use uuid::Uuid;

use config::Config;
use error::{Error, Result};
use http::{Http, HttpMethods};


/// Available Device Registry API methods.
pub trait RegistryApi {
    fn create_device(&mut Config, name: &str, id: &str, kind: DeviceType) -> Result<()>;
    fn delete_device(&mut Config, device: Uuid) -> Result<()>;
    fn list_device(&mut Config, device: Uuid) -> Result<()>;
    fn list_all_devices(&mut Config) -> Result<()>;

    fn create_group(&mut Config, name: &str) -> Result<()>;
    fn rename_group(&mut Config, group: Uuid, name: &str) -> Result<()>;
    fn add_to_group(&mut Config, group: Uuid, device: Uuid) -> Result<()>;
    fn remove_from_group(&mut Config, group: Uuid, device: Uuid) -> Result<()>;

    fn list_groups(&mut Config, device: Uuid) -> Result<()>;
    fn list_devices(&mut Config, group: Uuid) -> Result<()>;
    fn list_all_groups(&mut Config) -> Result<()>;
}


/// Make API calls to manage device groups.
pub struct Registry;

impl<'a> Registry {
    /// Parse CLI arguments into device listing preferences.
    pub fn list_device_flags(config: &mut Config, flags: &ArgMatches<'a>) -> Result<()> {
        match parse_list_flags(flags)? {
            (true, _, _) => Self::list_all_devices(config),
            (_, Some(device), _) => Self::list_device(config, device),
            (_, _, Some(group)) => Self::list_devices(config, group),
            _ => Err(Error::Flag("one of --all, --device, or --group required".into())),
        }
    }

    /// Parse CLI arguments into group listing preferences.
    pub fn list_group_flags(config: &mut Config, flags: &ArgMatches<'a>) -> Result<()> {
        match parse_list_flags(flags)? {
            (true, _, _) => Self::list_all_groups(config),
            (_, Some(device), _) => Self::list_groups(config, device),
            (_, _, Some(group)) => Self::list_devices(config, group),
            _ => Err(Error::Flag("one of --all, --device, or --group required".into())),
        }
    }
}

impl RegistryApi for Registry {
    fn create_device(config: &mut Config, name: &str, id: &str, kind: DeviceType) -> Result<()> {
        debug!("creating device {} of type {} with id {}", name, kind, id);
        let req = Client::new()
            .put(&format!("{}api/v1/devices", config.registry))
            .query(&[("deviceName", name), ("deviceId", id), ("kind", &format!("{}", kind))])
            .build()?;
        Http::send(req, config.token()?)
    }

    fn delete_device(config: &mut Config, device: Uuid) -> Result<()> {
        debug!("deleting device {}", device);
        Http::delete(&format!("{}api/v1/devices/{}", config.registry, device), config.token()?)
    }

    fn list_device(config: &mut Config, device: Uuid) -> Result<()> {
        debug!("listing details for device {}", device);
        Http::get(&format!("{}api/v1/devices/{}", config.registry, device), config.token()?)
    }

    fn list_all_devices(config: &mut Config) -> Result<()> {
        debug!("listing all devices");
        Http::get(&format!("{}api/v1/devices", config.registry), config.token()?)
    }

    fn create_group(config: &mut Config, name: &str) -> Result<()> {
        debug!("creating device group {}", name);
        let req = Client::new()
            .post(&format!("{}api/v1/device_groups", config.registry))
            .query(&[("groupName", name)])
            .build()?;
        Http::send(req, config.token()?)
    }

    fn rename_group(config: &mut Config, group: Uuid, name: &str) -> Result<()> {
        debug!("renaming group {} to {}", group, name);
        let req = Client::new()
            .put(&format!("{}api/v1/device_groups/{}/rename", config.registry, group))
            .query(&[("groupId", &format!("{}", group), ("groupName", name))])
            .build()?;
        Http::send(req, config.token()?)
    }

    fn add_to_group(config: &mut Config, group: Uuid, device: Uuid) -> Result<()> {
        debug!("adding device {} to group {}", device, group);
        let req = Client::new()
            .post(&format!("{}api/v1/device_groups/{}/devices/{}", config.registry, group, device))
            .query(&[("deviceId", device), ("groupId", group)])
            .build()?;
        Http::send(req, config.token()?)
    }

    fn remove_from_group(config: &mut Config, group: Uuid, device: Uuid) -> Result<()> {
        debug!("removing device {} from group {}", device, group);
        let req = Client::new()
            .delete(&format!("{}api/v1/device_groups/{}/devices/{}", config.registry, group, device))
            .query(&[("deviceId", format!("{}", device)), ("groupId", format!("{}", group))])
            .build()?;
        Http::send(req, config.token()?)
    }

    fn list_devices(config: &mut Config, group: Uuid) -> Result<()> {
        debug!("listing devices in group {}", group);
        Http::get(&format!("{}api/v1/device_groups/{}/devices", config.registry, group), config.token()?)
    }

    fn list_groups(config: &mut Config, device: Uuid) -> Result<()> {
        debug!("listing groups for device {}", device);
        Http::get(&format!("{}api/v1/devices/{}/groups", config.registry, device), config.token()?)
    }

    fn list_all_groups(config: &mut Config) -> Result<()> {
        debug!("listing all groups");
        Http::get(&format!("{}api/v1/device_groups", config.registry), config.token()?)
    }
}


/// Available device types.
#[derive(Clone, Copy, Debug)]
pub enum DeviceType {
    Vehicle,
    Other,
}

impl<'a> DeviceType {
    /// Parse CLI arguments into a `DeviceType`.
    pub fn from_flags(flags: &ArgMatches<'a>) -> Result<Self> {
        if flags.is_present("vehicle") {
            Ok(DeviceType::Vehicle)
        } else if flags.is_present("other") {
            Ok(DeviceType::Other)
        } else {
            Err(Error::Flag("Either --vehicle or --other flag is required".into()))
        }
    }
}

impl FromStr for DeviceType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "vehicle" => Ok(DeviceType::Vehicle),
            "other" => Ok(DeviceType::Other),
            _ => Err(Error::Parse(format!("unknown `DeviceType`: {}", s))),
        }
    }
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let text = match self {
            DeviceType::Vehicle => "Vehicle",
            DeviceType::Other => "Other",
        };
        write!(f, "{}", text)
    }
}


/// Parse into a tuple of --all, --device, and --group flags.
fn parse_list_flags<'a>(flags: &ArgMatches<'a>) -> Result<(bool, Option<Uuid>, Option<Uuid>)> {
    let all = flags.is_present("all");
    let device = if let Some(val) = flags.value_of("device") { Some(val.parse()?) } else { None };
    let group = if let Some(val) = flags.value_of("group") { Some(val.parse()?) } else { None };
    Ok((all, device, group))
}
