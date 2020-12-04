use udev::{Device, Event};

use crate::udevdetect::{DevFilter, get_device_property, get_event_property};

// FIXME: move to common area?
#[derive(Debug, Deserialize, Copy, Clone)]
pub enum USBHIDClass {
    Keyboard,
    Mouse,
}

#[derive(Debug)]
pub struct USBIDClassFilter {
    vendor: String,
    model: String,
    class: USBHIDClass,
}
impl USBIDClassFilter {
    pub fn new(vendor: u16, model: u16, class: USBHIDClass) -> Self {
        let f = USBIDClassFilter {
            vendor: format!("{:04x}", vendor),
            model: format!("{:04x}", model),
            class,
        };
        debug!("New USBIDClass Filter for: {:?} {:?}:{:?}", f.class, f.vendor, f.model);
        f
    }
}
impl DevFilter for USBIDClassFilter {
    fn match_event(&self, e: &Event) -> bool {
        if get_event_property(e, "ID_VENDOR_ID") != self.vendor {
            return false
        }
        if get_event_property(e, "ID_MODEL_ID") != self.model {
            return false
        }
        if get_event_property(e, get_class_prop(&self.class)) != "1" {
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
        if get_device_property(e, get_class_prop(&self.class)) != "1" {
            return false
        }
        true
    }
}

fn get_class_prop(c: &USBHIDClass) -> &str {
    match c {
        USBHIDClass::Keyboard => "ID_INPUT_KEYBOARD",
        USBHIDClass::Mouse => "ID_INPUT_MOUSE",
    }
}
