use anyhow::Result;
use evdev::{AttributeSet, EventType, InputEvent, Key};
use log::{debug, error, info, trace, warn};
use std::thread;

fn main() -> Result<()> {
    env_logger::builder().format_timestamp_millis().init();

    let mut threads = vec![];

    for device_name in ["Controller", "PCsensor FootSwitch Keyboard"] {
        info!("looking for device: {}", device_name);
        let Some((_, mut device)) =
            evdev::enumerate().find(|(_, device)| device.name().unwrap() == device_name)
        else {
            warn!("unknown device by name: {}", device_name);
            continue;
        };

        device.grab()?;
        debug!("successfully grabbed {:?}", device_name);

        let mut kbd = evdev::uinput::VirtualDeviceBuilder::new()?
            .name("mcro_kbd")
            .with_keys(&AttributeSet::from_iter(
                (0..Key::BTN_TRIGGER_HAPPY1.0).map(Key::new),
            ))?
            .build()?;

        let thread_handle = thread::spawn(move || -> Result<()> {
            info!(
                "listening to events by '{}': {:04x}:{:04x}",
                device.name().unwrap_or(device_name),
                device.input_id().vendor(),
                device.input_id().product(),
            );

            loop {
                for event in device.fetch_events()? {
                    trace!("{:?}", event);
                    match event.kind() {
                        evdev::InputEventKind::Key(key) => {
                            if key == Key::KEY_A || key == Key::BTN_WEST {
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
        });

        threads.push(thread_handle);
    }

    debug!(
        "setup a thread for one of each of the {} device(s)",
        threads.len()
    );

    for t in threads {
        if let Err(err) = t.join().expect("Failed to join on thread") {
            error!("{err}");
        };
    }

    Ok(())
}
