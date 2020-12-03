use std::path::Path;

use config::{Config, ConfigError, File};
use serde::export::fmt::Debug;

#[derive(Debug, Deserialize)]
pub struct SelfConfig {
    pub proxy_devices: Vec<ProxyDevice>,
    pub grab_devices: Vec<InputDevice>,
    pub log_level: String,
}

#[derive(Debug, Deserialize)]
pub struct ProxyDevice {
    pub name: String,
    pub vendor: u16,
    pub model: u16,
}

#[derive(Debug, Deserialize)]
pub struct InputDevice {
    pub name: String,
    pub selectors: Option<Vec<DeviceSelector>>,
}

#[derive(Debug, Deserialize)]
pub enum DeviceSelector {
    USBID{
        vendor: u16,
        model: u16
    },
}

pub fn read_config<P: AsRef<Path> + Debug + ToString>(path: P) -> Result<SelfConfig, ConfigError> {
    let mut c = Config::new();
    info!("Trying to read config from '{:?}'", path);
    c.merge(File::from(path.as_ref()))?;
    c.try_into()
}

