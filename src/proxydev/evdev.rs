use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;
use std::thread;

use crossbeam::channel::{Receiver, Sender};
use crossbeam::channel;
use input_linux::evdev::EvdevHandle;
use input_linux::sys::{input_event, timeval};

pub fn open_device<P: AsRef<Path> + Debug>(path: P) -> io::Result<EvdevHandle<File>> {
    let fd = OpenOptions::new().write(false).read(true).open(path)?;
    Ok(input_linux::evdev::EvdevHandle::new(fd))
}

pub fn device_poller<P: AsRef<Path> + Debug + ToString>(path: P, size: usize) -> io::Result<Receiver<input_event>> {
    let (tx, rx): (Sender<input_event>, Receiver<input_event>) = channel::bounded(size);

    let dev = open_device(&path)?;
    dev.grab(true)?;
    let path: String = path.to_string();
    thread::spawn(move || {
        let mut events: [input_event; 128] = [input_event{time:timeval{tv_usec:0,tv_sec:0},code:0,type_:0,value:0}; 128];
        loop {
            let res = match dev.read(&mut events) {
                Ok(ret) => ret,
                Err(err) => {
                    error!("Failed to read event for device '{:?}': {:?}", path, err);
                    return
                },
            };
            for ev in &events[..res] {
                match tx.send(*ev) {
                    Ok(_) => {
                        debug!("Sent message from '{:}'", path);
                    },
                    Err(err) => {
                        error!("Failed to send event for device '{:?}', dropping: {:?}", path, err);
                        return
                    },
                };
            }
        }
    });

    Ok(rx)
}

