use std::path::Path;

use config::{Config, ConfigError, File};
use serde::export::fmt::Debug;

use super::udevdetect::USBHIDClass;
use super::proxydev::SimpleDeviceClass;

#[derive(Debug, Deserialize)]
pub struct SelfConfig {
    pub device: Vec<Device>,
    // pub grab_devices: Vec<InputDevice>,
    pub log_level: String,
}

#[derive(Debug, Deserialize)]
pub enum Device {
    Simple {
        name: String,
        vendor: u16,
        model: u16,
        class: SimpleDeviceClass,
        selector: Option<Vec<DeviceSelector>>,
    },
}

#[derive(Debug, Deserialize)]
pub enum DeviceSelector {
    USBID{
        vendor: u16,
        model: u16
    },
    USBIDClass{
        vendor: u16,
        model: u16,
        class: USBHIDClass,
    },
}

pub fn read_config<P: AsRef<Path> + Debug + ToString>(path: P) -> Result<SelfConfig, ConfigError> {
    let mut c = Config::new();
    info!("Trying to read config from '{:?}'", path);
    c.merge(File::from(path.as_ref()))?;
    c.try_into()
}

