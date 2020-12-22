#[macro_use] extern crate clap;
#[macro_use] extern crate log;
extern crate pretty_env_logger;
#[macro_use] extern crate serde_derive;

use std::env;
use std::thread;

use clap::Arg;

mod udevdetect;
mod proxydev;
mod config;

static DEFAULT_CONFIG_PATH: &str = "/etc/evdev-proxy/config.toml";

fn selector_by_config(s: &config::DeviceSelector) -> Box<dyn udevdetect::DevFilter+Send+Sync> {
    match s {
        config::DeviceSelector::USBID{vendor, model} => {
            Box::new(udevdetect::USBIDFilter::new(*vendor, *model))
        },
        config::DeviceSelector::USBIDClass{vendor, model, class} => {
            Box::new(udevdetect::USBIDClassFilter::new(*vendor, *model, *class))
        }
    }
}

fn main() {
    let app = clap::App::new("evdev-proxy")
        .about("Creates virtual devices to proxy other evdev devices with hotplug support")
        .version(crate_version!())
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .about("Path to config file")
            .takes_value(true)).get_matches();

    let config_path = app.value_of("config").unwrap_or(DEFAULT_CONFIG_PATH);
    let conf = config::read_config(config_path)
        .expect("Failed to read config file");

    match env::var("RUST_LOG") {
        Err(_) => {
            info!("RUST_LOG not set, using log-level from config");
            env::set_var("RUST_LOG", &conf.log_level);
        },
        Ok(_) => (),
    }
    pretty_env_logger::init();
    info!("Parsed config: {:#?}", conf);

    let mut threads = Vec::new();
    for dev in conf.device {
        match dev {
            config::Device::Simple{name, vendor, model, class, selector} => {
                let t = thread::spawn(move || {
                    // create simple proxy device
                    let pd = proxydev::Simple::new(name.as_str(), class, vendor, model, )
                        .expect("Failed to create proxy device");
                    info!("Proxy device initialized as '{:?}'", pd.dev_path());

                    // create udev listener with device selectors
                    info!("Initializing udev listener for '{:?}'", name);
                    let mut dl = udevdetect::DevListener::new("input", 32);
                    for s in selector.as_ref().unwrap_or(&Vec::new()) {
                        let filter = selector_by_config(s);
                        dl.add_filter(filter);
                    }
                    let dev_ev_listener = dl.listen()
                        .expect("Failed to listen to udev events");

                    info!("Listening for udev events for '{:?}'", name);
                    for event in dev_ev_listener.iter() {
                        info!("Device event for {:?}: {:?}", name, event);
                        if event.action == udevdetect::DevEventType::Add {
                            info!("Got matching device event for '{:?}': {:?}:{:?} ({:?})",
                                  name, event.vendor, event.product, event.input_class);

                            match pd.add_source_dev(&event.devname) {
                                Err(e) => {
                                    error!("Failed to add matched device '{:?}': {:?}", event.devname, e);
                                },
                                Ok(_) => {},
                            };
                        }
                        info!("Number of devices: {:}", pd.num_sources());
                    }
                    info!("Thread for device '{:?}' has finished", name);
                });
                threads.push(t);
            }
        }
    }

    // Wait for all threads
    for t in threads {
        t.join().unwrap();
    }
}

