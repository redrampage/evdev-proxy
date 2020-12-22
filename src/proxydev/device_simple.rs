use std::fmt::Debug;
use std::io;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam::channel::{Receiver, Sender};
use crossbeam::channel;
use input_linux::sys::input_event;

use crate::proxydev::evdev::device_poller;
use crate::proxydev::uinput::{new_uinput_aio, new_uinput_kbd, new_uinput_mouse};

#[derive(Debug)]
pub struct Simple {
    name: String,
    sources: Arc<Mutex<Vec<(String, Receiver<input_event>)>>>,
    ch_reload: (Sender<bool>, Receiver<bool>),
    devpath: String,
}

#[derive(Debug, Deserialize)]
pub enum SimpleDeviceClass {
    Keyboard,
    Mouse,
    AIO,
}

impl Simple {
    pub fn new(name: &str, class: SimpleDeviceClass, vendor: u16, model: u16) -> io::Result<Simple> {
        info!("Creating new simple proxy device '{:?}' ({:04x}:{:04x})", name, vendor, model);
        let uin = match class {
            SimpleDeviceClass::Keyboard => {
                new_uinput_kbd(name, vendor, model)?
            },
            SimpleDeviceClass::Mouse => {
                new_uinput_mouse(name, vendor, model)?
            },
            SimpleDeviceClass::AIO => {
                new_uinput_aio(name, vendor, model)?
            },
        };

        let dev = Simple {
            name: name.to_owned(),
            sources: Arc::new(Mutex::new(Vec::new())),
            ch_reload: channel::bounded(1),
            devpath: uin.evdev_path().unwrap().into_os_string().into_string().unwrap(),
        };

        // Those vars if for thread
        let sources = dev.sources.clone();
        let ch_reload = dev.ch_reload.1.clone();
        let dev_name = name.to_owned();
        thread::spawn(move || {
            info!("Starting event loop for proxy device '{:}'", dev_name);
            'device: loop { // Initialization loop
                let mut event_selector = channel::Select::new();
                let mut local_sources;

                // Copy all sources to thread-local vector
                {
                    let psrc = &sources.lock().unwrap();
                    local_sources = Vec::with_capacity(psrc.len());
                    for (name, s) in psrc.iter() {
                        local_sources.push((name.to_owned(), s.clone()));
                    }
                }

                // Populate event selector with all sources and reload signal channel
                for (_, src) in &local_sources {
                    event_selector.recv(&src);
                }
                event_selector.recv(&ch_reload);

                // Source/reload event loop
                loop {
                    let op = event_selector.select();
                    let op_idx = op.index();

                    if op_idx == local_sources.len() {
                        // Got reload signal
                        info!("Reloading event loop for proxy device '{:?}'", dev_name);
                        op.recv(&ch_reload).unwrap();
                        continue 'device;
                    }

                    let (n, rx) = &local_sources[op_idx];
                    let ev = match op.recv(rx) {
                        Ok(e) => e,
                        Err(_) => {
                            error!("Failed to read source device '{:?}', removing from '{:?}' and reloading", n, dev_name);
                            remove_dev_from_list(&sources, n);
                            continue 'device
                        }
                    };

                    debug!("Proxy device '{:?}' got event from '{:?}': {:?}", dev_name, n, ev);
                    match uin.write(&[ev]) {
                        Ok(_) => {},
                        Err(e) => {
                            error!("Failed to write event to '{:?}': {:?}", dev_name, e);
                        }
                    }
                }
            }
        });

        Ok(dev)
    }

    pub fn add_source_dev<P: AsRef<Path> + Debug + ToString>(&self, path: P) -> io::Result<()> {
        let rx = device_poller(path.to_string(), 64)?;
        match self.sources.lock(){
            Ok(mut srcs) => {
                info!("Added new source dev '{:?}'", path);
                srcs.push((path.to_string(), rx));
                // Careful here, we're still holding sources mutex and trying to send reload signal
                self.ch_reload.0.send(true).unwrap();
                Ok(())
            },
            Err(err) => {
                error!("Failed to add source dev: {:?}", err);
                Err(io::Error::new(ErrorKind::Other, err.to_string()))
            }
        }
    }

    pub fn num_sources(&self) -> usize {
        self.sources.lock().unwrap().len()
    }

    pub fn dev_path(&self) -> &str {
        &self.devpath
    }
}

fn remove_dev_from_list<P: AsRef<Path> + Debug + ToString, T>(list: &Arc<Mutex<Vec<(String, T)>>>, path: P) {
    let mut s = list.lock().unwrap();
    match s.iter().position(|(name, _)| {name == &path.to_string()}) {
        None => debug!("Failed to remove element '{:?}'", path),
        Some(idx) => {s.remove(idx);},
    };
}

// fn copy_sources<T: ToOwned, Y: Clone>(sources: &Mutex<Vec<(T, Y)>>) -> Vec<(T, Y)> {
//     let psrc = &sources.lock().unwrap();
//     let local_sources = Vec::with_capacity(psrc.len());
//     for (name, s) in psrc.iter() {
//         local_sources.push((name.to_owned(), s.clone()));
//     }
// }

