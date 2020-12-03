use udev::{Device, Event};

use crate::udevdetect::{DevFilter, get_device_property, get_event_property};

#[derive(Debug)]
pub struct DevIDFilter {
    vendor: String,
    model: String,
}
impl DevIDFilter {
    pub fn new(vendor: u16, model: u16) -> Self {
        let f = DevIDFilter{
            vendor: format!("{:04x}", vendor),
            model: format!("{:04x}", model),
        };
        debug!("New DEVID Filter for: {:?}:{:?}", f.vendor, f.model);
        f
    }
}
impl DevFilter for DevIDFilter {
    fn match_event(&self, e: &Event) -> bool {
        if get_event_property(e, "ID_VENDOR_ID") != self.vendor {
            return false
        }
        if get_event_property(e, "ID_MODEL_ID") != self.model {
            return false
        }
        true
    }

    fn match_device(&self, e: &Device) -> bool {
        if get_device_property(e, "ID_VENDOR_ID") != self.vendor {
            return false
        }
        if get_device_property(e, "ID_MODEL_ID") != self.model {
            return false
        }
        true
    }
}
