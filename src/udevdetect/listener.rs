use std::io;
use std::io::Error;
use std::os::unix::io::AsRawFd;
use std::thread;

use crossbeam::channel::{Receiver, Sender};
use crossbeam::channel;
use nix::poll::{PollFd, PollFlags, ppoll};
use nix::sys::signal::SigSet;

use crate::udevdetect::{get_device_property, get_event_property};
use crate::udevdetect::filter::DevFilter;

pub struct DevListener {
    filters: Vec<Box<dyn DevFilter+Send+Sync>>,
    subsystem: String,
    event_queue_size: usize,
}

#[derive(Debug,PartialEq)]
pub enum DevEventType {
    UNKNOWN,
    Add,
    Remove,
}

impl From<&str> for DevEventType {
    fn from(action_raw: &str) -> Self {
        if action_raw == "add" {
            DevEventType::Add
        } else if action_raw == "remove" {
            DevEventType::Remove
        } else {
            DevEventType::UNKNOWN
        }
    }
}

#[derive(Debug)]
pub struct DevEvent {
    // capabilities: HashMap<String, String>,
    pub action: DevEventType,
    pub vendor: String,
    pub product: String,
    pub name: String,
    pub input_class: String,
    pub devpath: String,
    pub devname: String,
}

impl DevListener {
    pub fn new(subsystem: &str, queue_size: usize) -> DevListener {
        DevListener{
            subsystem: subsystem.to_owned(),
            filters: Vec::new(),
            event_queue_size: queue_size,
        }
    }

    pub fn add_filter(&mut self, filter: Box<dyn DevFilter+Send+Sync>) {
        self.filters.push(filter);
    }

    pub fn listen(self) -> io::Result<Receiver<DevEvent>> {
        let (sender, receiver): (Sender<DevEvent>, Receiver<DevEvent>) = channel::bounded(self.event_queue_size);

        info!("Listing present devices for subsystem '{:}'", self.subsystem);
        let mut enumerator = udev::Enumerator::new().unwrap();
        enumerator.match_subsystem(&self.subsystem).unwrap();
        let dev_iter = enumerator.scan_devices().unwrap();
        let devices: Vec<DevEvent> = dev_iter.filter_map(|dev| -> Option<DevEvent> {
            // Skip devices without node
            if get_device_property(&dev, "DEVNAME") == "" {
                debug!("Skipping device '{:?}', no devname property", dev.syspath());
                return None
            }

            let mut matched = false;
            for f in self.filters.iter() {
                if f.match_device(&dev) {
                    matched = true;
                }
            }
            if !matched {
                debug!("Skipping device '{:?}, do not match any filters", dev.syspath());
                return None
            }

            let devpath_raw = get_device_property(&dev, "DEVPATH");
            let devname_raw = get_device_property(&dev, "DEVNAME");
            let input_class_raw = get_device_property(&dev, ".INPUT_CLASS");
            let name_raw = get_device_property(&dev, "NAME");
            let vendor_raw = get_device_property(&dev, "ID_VENDOR_ID");
            let model_raw = get_device_property(&dev, "ID_MODEL_ID");

            // println!("driver: {:?}", dev.driver());
            // for at in dev.attributes() {
            //     println!("ATTR: {:?} = {:?}", at.name(), at.value());
            // }
            // for prop in dev.properties() {
            //     println!("PROP: {:?} = {:?}", prop.name(), prop.value());
            // }


            Some(DevEvent{
                action: DevEventType::Add,
                devpath: devpath_raw.to_owned(),
                devname: devname_raw.to_owned(),
                input_class: input_class_raw.to_owned(),
                name: name_raw.to_owned(),
                vendor: vendor_raw.to_owned(),
                product: model_raw.to_owned(),
            })
        }).collect();

        {
            let devices = devices;
            let sender = sender.clone();

            thread::spawn(move || {
                for d in devices {
                    sender.send(d).unwrap();
                }
            });
        }

        info!("Spawning listening thread for sybsystem '{:}'", self.subsystem);
        {
            let (err_sender, err_receiver): (Sender<Error>, Receiver<Error>) = channel::bounded(0);

            // let mut err_sender = err_sender.clone();
            let sender = sender.clone();
            let subsystem = self.subsystem.to_owned();
            let filters = self.filters;

            thread::spawn(move || {
                let mut udevmonbuilder;
                let mut udevmon;
                let fd;
                {
                    let err = err_sender;
                    udevmonbuilder = match udev::MonitorBuilder::new() {
                        Ok(m) => m,
                        Err(e) => {
                            err.send(e).unwrap();
                            return
                        },
                    };
                    udevmonbuilder = match udevmonbuilder.match_subsystem(subsystem) {
                        Ok(m) => m,
                        Err(e) => {
                            err.send(e).unwrap();
                            return
                        },
                    };
                    udevmon = match udevmonbuilder.listen() {
                        Ok(m) => m,
                        Err(e) => {
                            err.send(e).unwrap();
                            return
                        },
                    };
                    fd = PollFd::new(udevmon.as_raw_fd(), PollFlags::POLLIN);
                };

                'event: loop {
                    ppoll(&mut [fd], None, SigSet::empty()).unwrap();

                    let event = match udevmon.next() {
                        Some(evt) => evt,
                        None => {
                            // sleep(Duration::from_millis(10));
                            continue 'event;
                        }
                    };

                    let action_raw = get_event_property(&event, "ACTION");
                    let devpath_raw = get_event_property(&event, "DEVPATH");
                    let devname_raw = get_event_property(&event, "DEVNAME");
                    let input_class_raw = get_event_property(&event, ".INPUT_CLASS");
                    let name_raw = get_event_property(&event, "NAME");
                    let vendor_raw = get_event_property(&event, "ID_VENDOR_ID");
                    let model_raw = get_event_property(&event, "ID_MODEL_ID");

                    // Skip devices without node
                    if devname_raw == "" {
                        debug!("Skipping event for '{:?}', no devname property", event.syspath());
                        continue 'event
                    }

                    let mut matched = false;
                    for f in filters.iter() {
                        if f.match_event(&event) {
                            matched = true;
                        }
                    }
                    if !matched {
                        debug!("Skipping event for '{:?}, do not match any filters", event.syspath());
                        continue 'event
                    }

                    debug!("Emitting event for '{:?}'", event.syspath());
                    let dev_event = DevEvent {
                        action: DevEventType::from(action_raw),
                        devpath: devpath_raw.to_owned(),
                        devname: devname_raw.to_owned(),
                        input_class: input_class_raw.to_owned(),
                        name: name_raw.to_owned(),
                        vendor: vendor_raw.to_owned(),
                        product: model_raw.to_owned(),
                    };

                    match sender.send(dev_event) {
                        Err(e) => {
                            error!("Failed to send udev event for '{:?}': {:?}", event.syspath(), e);
                        }
                        Ok(_) => {},
                    }
                }
            });

            // Check for error after event listener init
            match err_receiver.recv() {
                Ok(err) => return Err(err),
                Err(_) => {},
            };
        }

        std::mem::drop(sender);
        Ok(receiver)
    }
}
