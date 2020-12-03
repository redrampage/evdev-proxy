#[macro_use] extern crate log;
extern crate pretty_env_logger;
#[macro_use] extern crate serde_derive;

use crate::udevdetect::DevIDFilter;
use std::env;

mod udevdetect;
mod proxydev;
mod config;

// FIXME: get config path from arguments
static CONFIG_PATH: &str = "/etc/evdev-proxy/config.toml";

fn main() {
    let conf = config::read_config(CONFIG_PATH)
        .expect("Failed to read config file");

    match env::var("RUST_LOG") {
        Err(_) => {
            info!("RUST_LOG not set, using log-level from config");
            env::set_var("RUST_LOG", &conf.log_level);
        },
        Ok(_) => (),
    }
    pretty_env_logger::init();
    debug!("Parsed config: {:#?}", conf);

    // FIXME!!!
    if conf.proxy_devices.len() > 1 {
        panic!("Only one proxy device currently supported");
    } else if conf.proxy_devices.len() == 0 {
        panic!("No proxy devices specified");
    }

    // FIXME!!!
    if conf.grab_devices.len() > 1 {
        panic!("Only one grab device currently supported");
    } else if conf.proxy_devices.len() == 0 {
        panic!("No grab devices specified");
    }

    let pdconf = conf.proxy_devices.first()
        .expect("Failed to get proxy device config");
    let pd = proxydev::ProxyDev::new(pdconf.name.as_str(), pdconf.vendor, pdconf.model)
        .expect("Failed to create proxy device");
    info!("Proxy devices initialized");

    info!("Initializing udev listener");
    let gdconf = conf.grab_devices.first()
        .expect("Failed to get grab device config");
    let mut dl = udevdetect::DevListener::new("input", 32);
    for selector in gdconf.selectors.as_ref().unwrap_or(&Vec::new()) {
        match selector {
            config::DeviceSelector::USBID{vendor, model} => {
                dl.add_filter(Box::new(DevIDFilter::new(*vendor, *model)))
            }
        }
    }
    let iter = dl.listen()
        .expect("Failed to listen to udev events");

    info!("Listening for udev events");
    for event in iter.iter() {
        info!("GOT EVENT {:?}", event);

        if event.action == udevdetect::DevEventType::Add {
            match pd.add_source_dev(&event.devname) {
                Err(e) => {
                    error!("Failed to add matched device '{:?}': {:?}", event.devname, e);
                    continue
                },
                Ok(_) => {},
            };
        }
        info!("Number of devices: {:}", pd.num_sources());
    }
    info!("Main thread finished");
}

