use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

use input_linux::{EventKind, InputId, Key, MiscKind, RelativeAxis, UInputHandle};
use input_linux::sys;

static DEFAULT_KEYS: [Key; 106] = [
    // Row 1
    Key::Esc,

    Key::F1,
    Key::F2,
    Key::F3,
    Key::F4,
    Key::F5,
    Key::F6,
    Key::F7,
    Key::F8,
    Key::F9,
    Key::F10,
    Key::F11,
    Key::F12,

    Key::Print,
    Key::Sysrq,
    Key::ScrollLock,
    Key::Pause,
    Key::Break,

    // Row 2
    Key::Grave,
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
    Key::Num0,
    Key::Minus,
    Key::Equal,
    Key::Backspace,

    Key::Insert,
    Key::Home,
    Key::PageUp,

    Key::NumLock,
    Key::KpSlash,
    Key::KpAsterisk,
    Key::KpMinus,

    // Row 3
    Key::Tab,
    Key::Q,
    Key::W,
    Key::E,
    Key::R,
    Key::T,
    Key::Y,
    Key::U,
    Key::I,
    Key::O,
    Key::P,
    Key::LeftBrace,
    Key::RightBrace,
    Key::Backslash,

    Key::Delete,
    Key::End,
    Key::PageDown,

    Key::Kp7,
    Key::Kp8,
    Key::Kp9,
    Key::KpPlus,

    // Row 4
    Key::CapsLock,
    Key::A,
    Key::S,
    Key::D,
    Key::F,
    Key::G,
    Key::H,
    Key::J,
    Key::K,
    Key::L,
    Key::Semicolon,
    Key::Apostrophe,
    Key::Enter,

    Key::Kp4,
    Key::Kp5,
    Key::Kp6,

    // Row 5
    Key::LeftShift,
    Key::Z,
    Key::X,
    Key::C,
    Key::V,
    Key::B,
    Key::N,
    Key::M,
    Key::Comma,
    Key::Dot,
    Key::Slash,
    Key::RightShift,

    Key::Up,

    Key::Kp1,
    Key::Kp2,
    Key::Kp3,
    Key::KpEnter,

    // Row 6
    Key::LeftCtrl,
    Key::LeftMeta,
    Key::LeftAlt,
    Key::Space,
    Key::RightAlt,
    Key::Fn,
    Key::Menu,
    Key::RightCtrl,

    Key::Left,
    Key::Down,
    Key::Right,

    Key::Kp0,
    Key::KpDot,

];

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

pub fn new_uinput_device<P: AsRef<Path> + Debug>(path: P, name: &str, vendor: u16, product: u16) -> io::Result<UInputHandle<File>> {
    info!("Creating UInput device '{:}' ({:x}:{:x})", name, vendor, product);

    let id = InputId{
         vendor,
         product,
         bustype: sys::BUS_USB,
         version: 5,
    };

    let fd = OpenOptions::new().read(true).write(true).open(path)?;
    let handle = UInputHandle::new(fd);

    for t in &[EventKind::Synchronize, EventKind::Misc, EventKind::Key,
            EventKind::Relative] {
        debug!("Setting EvKindBit flag: {:?}", t);
        handle.set_evbit(*t)?;
    }

    for m in &[MiscKind::Scancode] {
        handle.set_mscbit(*m)?;
    }

    for k in &DEFAULT_KEYS {
        debug!("Setting KeyBit flag: {:?}", k);
        handle.set_keybit(*k)?;
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
    info!("UInput device '{:?}'({:?}) successfully created", handle.sys_path()?, handle.evdev_name()?);
    Ok(handle)
}
