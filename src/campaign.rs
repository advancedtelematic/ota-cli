use failure::Error;
use reqwest::Client;
use std::path::PathBuf;
use uuid::Uuid;


pub trait Campaign {
    fn create(&self, campaign_id: Uuid, name: &str, groups: Vec<&str>) -> Result<(), Error>;
    fn get(&self, campaign_id: Uuid) -> Result<(), Error>;
    fn launch(&self, campaign_id: Uuid) -> Result<(), Error>;
    fn cancel(&self, campaign_id: Uuid) -> Result<(), Error>;
}

pub struct Manager {
    client: Client,
    credentials: PathBuf,
}

impl Manager {
    pub fn new(credentials: PathBuf) -> Self {
        Manager {
            client: Client::new(),
            credentials,
        }
    }
}

impl Campaign for Manager {
    fn create(&self, campaign_id: Uuid, name: &str, groups: Vec<&str>) -> Result<(), Error> {
        unimplemented!()
    }

    fn get(&self, campaign_id: Uuid) -> Result<(), Error> { unimplemented!() }

    fn launch(&self, campaign_id: Uuid) -> Result<(), Error> { unimplemented!() }

    fn cancel(&self, campaign_id: Uuid) -> Result<(), Error> { unimplemented!() }
}
