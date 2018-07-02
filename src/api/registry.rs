use clap::ArgMatches;
use uuid::Uuid;

use config::Config;
use error::{Error, Result};
use util::print_resp;


/// Available Device Registry API methods.
pub trait RegistryApi {
    fn create_group(&mut Config, name: &str) -> Result<()>;
    fn rename_group(&mut Config, group: Uuid, name: &str) -> Result<()>;

    fn add_to_group(&mut Config, group: Uuid, device: Uuid) -> Result<()>;
    fn remove_from_group(&mut Config, group: Uuid, device: Uuid) -> Result<()>;

    fn list_devices(&mut Config, group: Uuid) -> Result<()>;
    fn list_groups(&mut Config, device: Uuid) -> Result<()>;
    fn list_all_groups(&mut Config) -> Result<()>;
}


/// Make API calls to manage device groups.
pub struct Registry;

impl<'a> Registry {
    pub fn delegate(config: &mut Config, matches: &ArgMatches<'a>) -> Result<()> {
        if let Some(group) = matches.value_of("group") {
            Self::list_devices(config, group.parse()?)
        } else if let Some(device) = matches.value_of("device") {
            Self::list_groups(config, device.parse()?)
        } else if matches.is_present("all") {
            Self::list_all_groups(config)
        } else {
            Err(Error::Flag("one of --all, --group or --device flag required".into()))
        }
    }
}

impl RegistryApi for Registry {
    fn create_group(config: &mut Config, name: &str) -> Result<()> {
        debug!("creating device group {}", name);
        config
            .client()
            .post(&format!("{}api/v1/device_groups", config.registry))
            .query(&[("groupName", name)])
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn rename_group(config: &mut Config, group: Uuid, name: &str) -> Result<()> {
        debug!("renaming group {} to {}", group, name);
        config
            .client()
            .put(&format!("{}api/v1/device_groups/{}/rename", config.registry, group))
            .query(&[("groupId", format!("{}", group).as_ref()), ("groupName", name)])
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn add_to_group(config: &mut Config, group: Uuid, device: Uuid) -> Result<()> {
        debug!("adding device {} to group {}", device, group);
        config
            .client()
            .post(&format!("{}api/v1/device_groups/{}/devices/{}", config.registry, device, group))
            .query(&[("deviceId", device), ("groupId", group)])
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn remove_from_group(config: &mut Config, group: Uuid, device: Uuid) -> Result<()> {
        debug!("removing device {} from group {}", device, group);
        config
            .client()
            .delete(&format!("{}api/v1/device_groups/{}/devices/{}", config.registry, device, group))
            .query(&[("deviceId", device), ("groupId", group)])
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn list_devices(config: &mut Config, group: Uuid) -> Result<()> {
        debug!("listing devices in group {}", group);
        config
            .client()
            .get(&format!("{}api/v1/device_groups/{}/devices", config.registry, group))
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn list_groups(config: &mut Config, device: Uuid) -> Result<()> {
        debug!("listing groups for device {}", device);
        config
            .client()
            .get(&format!("{}api/v1/devices/{}/groups", config.registry, device))
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }

    fn list_all_groups(config: &mut Config) -> Result<()> {
        debug!("listing all groups");
        config
            .client()
            .get(&format!("{}api/v1/device_groups", config.registry))
            .headers(config.bearer_token()?)
            .send()
            .map_err(Error::Http)
            .and_then(print_resp)
    }
}
