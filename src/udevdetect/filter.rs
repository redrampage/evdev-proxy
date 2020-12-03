use std::fmt::Debug;

use udev::{Device, Event};

pub trait DevFilter: Debug {
    fn match_event(&self, e: &Event) -> bool;
    fn match_device(&self, e: &Device) -> bool;
}


