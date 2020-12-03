use std::ffi::OsStr;

use udev::{Device, Event};

pub use self::listener::DevEvent;
pub use self::listener::DevEventType;
pub use self::listener::DevListener;
pub use self::filter::DevFilter;
pub use self::filter_dev_id::DevIDFilter;

mod listener;
mod filter;
mod filter_dev_id;

fn get_event_property<'a>(ev: &'a Event, key: &'a str) -> &'a str {
    ev.property_value(key).unwrap_or(OsStr::new("")).to_str().unwrap_or("")
}

fn get_device_property<'a>(ev: &'a Device, key: &'a str) -> &'a str {
    ev.property_value(key).unwrap_or(OsStr::new("")).to_str().unwrap_or("")
}

