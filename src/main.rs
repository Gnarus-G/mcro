use std::error::Error;

use evdev::{AttributeSet, EventType, InputEvent, Key};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let Some((_, mut device)) = evdev::enumerate()
        .find(|(_, device)| device.name().unwrap() == "PCsensor FootSwitch Keyboard")
    else {
        return Ok(());
    };

    device.grab()?;

    let mut kbd = evdev::uinput::VirtualDeviceBuilder::new()?
        .name("mcro_kbd")
        .with_keys(&AttributeSet::from_iter(
            (0..Key::BTN_TRIGGER_HAPPY1.0).map(Key::new),
        ))?
        .build()?;

    loop {
        for event in device.fetch_events()? {
            eprintln!("[DEBUG] {:?}", event);

            match event.kind() {
                evdev::InputEventKind::Key(key) => {
                    if key == Key::KEY_A {
                        kbd.emit(&[
                            InputEvent::new(
                                EventType::KEY,
                                Key::KEY_LEFTSHIFT.code(),
                                event.value().min(1),
                            ),
                            InputEvent::new(
                                EventType::KEY,
                                Key::KEY_LEFTALT.code(),
                                event.value().min(1),
                            ),
                            InputEvent::new(
                                EventType::KEY,
                                Key::KEY_RIGHTCTRL.code(),
                                event.value().min(1),
                            ),
                        ])?;
                    }
                }
                evdev::InputEventKind::Synchronization(_) => {
                    kbd.emit(&[event])?;
                }
                _ => {}
            };
        }
    }
}
