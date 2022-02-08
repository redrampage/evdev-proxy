use std::fs::{File, OpenOptions};
use std::io;

use input_linux::{EventKind, InputId, Key, MiscKind, RelativeAxis, UInputHandle};
use input_linux::sys;

static MOUSE_KEYS: [Key; 18] = [
    // Mouse buttons
    Key::Button0,
    Key::Button1,
    Key::Button2,
    Key::Button3,
    Key::Button4,
    Key::Button5,
    Key::Button6,
    Key::Button7,
    Key::Button8,
    Key::Button9,

    Key::ButtonLeft,
    Key::ButtonRight,
    Key::ButtonMiddle,
    Key::ButtonSide,
    Key::ButtonExtra,
    Key::ButtonForward,
    Key::ButtonBack,
    Key::ButtonTask,
];

static UINPUT_PATH: &str = "/dev/uinput";

pub fn new_uinput_kbd(name: &str, vendor: u16, product: u16) -> io::Result<UInputHandle<File>> {
    info!("Creating new uinput keyboard");
    let id = InputId{
        vendor,
        product,
        bustype: sys::BUS_USB,
        version: 5,
    };
    let fd = OpenOptions::new().read(true).write(true).open(UINPUT_PATH)?;
    let handle = UInputHandle::new(fd);

    for t in &[EventKind::Synchronize, EventKind::Misc, EventKind::Key] {
        debug!("Setting EvKindBit flag: {:?}", t);
        handle.set_evbit(*t)?;
    }

    for m in &[MiscKind::Scancode] {
        handle.set_mscbit(*m)?;
    }

    for k in Key::iter() {
        debug!("Setting KeyBit flag: {:?}", k);
        handle.set_keybit(k)?;
    }

    handle.create(&id, name.as_bytes(), 0, &vec![])?;
    info!("UInput keyboard device '{:?}'({:?}) successfully created", handle.sys_path()?, handle.evdev_name()?);
    Ok(handle)
}

pub fn new_uinput_mouse(name: &str, vendor: u16, product: u16) -> io::Result<UInputHandle<File>> {
    info!("Creating new uinput mouse");
    let id = InputId{
        vendor,
        product,
        bustype: sys::BUS_USB,
        version: 5,
    };
    let fd = OpenOptions::new().read(true).write(true).open(UINPUT_PATH)?;
    let handle = UInputHandle::new(fd);

    for t in &[EventKind::Synchronize, EventKind::Misc, EventKind::Key,
        EventKind::Relative] {
        debug!("Setting EvKindBit flag: {:?}", t);
        handle.set_evbit(*t)?;
    }

    for m in &[MiscKind::Scancode] {
        handle.set_mscbit(*m)?;
    }

    for k in &MOUSE_KEYS {
        debug!("Setting KeyBit flag: {:?}", k);
        handle.set_keybit(*k)?;
    }

    // Mouse AXIS
    for r in &[RelativeAxis::X, RelativeAxis::Y, RelativeAxis:: Wheel,
        RelativeAxis::HorizontalWheel,
        RelativeAxis::WheelHiRes, RelativeAxis::HorizontalWheelHiRes] {
        debug!("Setting Relative Axis flag: {:?}", r);
        handle.set_relbit(*r)?;
    }

    handle.create(&id, name.as_bytes(), 0, &vec![])?;
    info!("UInput mouse device '{:?}'({:?}) successfully created", handle.sys_path()?, handle.evdev_name()?);
    Ok(handle)
}

pub fn new_uinput_aio(name: &str, vendor: u16, product: u16) -> io::Result<UInputHandle<File>> {
    info!("Creating AIO UInput device '{:}' ({:x}:{:x})", name, vendor, product);

    let id = InputId{
         vendor,
         product,
         bustype: sys::BUS_USB,
         version: 5,
    };

    let fd = OpenOptions::new().read(true).write(true).open(UINPUT_PATH)?;
    let handle = UInputHandle::new(fd);

    for t in &[EventKind::Synchronize, EventKind::Misc, EventKind::Key,
            EventKind::Relative] {
        debug!("Setting EvKindBit flag: {:?}", t);
        handle.set_evbit(*t)?;
    }

    for m in &[MiscKind::Scancode] {
        handle.set_mscbit(*m)?;
    }

    for k in Key::iter() {
        debug!("Setting KeyBit flag: {:?}", k);
        handle.set_keybit(k)?;
    }
    for k in &MOUSE_KEYS {
        debug!("Setting Mouse KeyBit flag: {:?}", k);
        handle.set_keybit(*k)?;
    }

    // Mouse AXIS
    for r in &[RelativeAxis::X, RelativeAxis::Y, RelativeAxis:: Wheel,
            RelativeAxis::HorizontalWheel,
            RelativeAxis::WheelHiRes, RelativeAxis::HorizontalWheelHiRes] {
        debug!("Setting Relative Axis flag: {:?}", r);
        handle.set_relbit(*r)?;
    }

    handle.create(&id, name.as_bytes(), 0, &vec![])?;
    info!("AIO UInput device '{:?}'({:?}) successfully created", handle.sys_path()?, handle.evdev_name()?);
    Ok(handle)
}
