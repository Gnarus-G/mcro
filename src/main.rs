use anyhow::Result;
use evdev::{AttributeSet, EventType, InputEvent, Key};
use log::{debug, error, info, trace, warn};
use std::thread;

// --- Functional Core (pure, testable) ---
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InEvent {
    Key { key: Key, value: i32 },
    Sync,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Action {
    // Keys to emit with their values (-1 release, 0 repeat, 1 press)
    EmitKeys(Vec<(Key, i32)>),
    // Emit a synchronization event
    EmitSync,
}

fn clamp_key_value(v: i32) -> i32 {
    // Kernel sometimes reports >1 for press; we clamp to 1 to stay deterministic
    if v > 1 {
        1
    } else {
        v
    }
}

fn transform(event: InEvent) -> Option<Action> {
    match event {
        InEvent::Key { key, value } if key == Key::KEY_A || key == Key::BTN_WEST => {
            let v = clamp_key_value(value);
            Some(Action::EmitKeys(vec![
                (Key::KEY_LEFTSHIFT, v),
                (Key::KEY_LEFTALT, v),
                (Key::KEY_RIGHTCTRL, v),
            ]))
        }
        InEvent::Key {
            key: Key::KEY_B,
            value,
        } => Some(Action::EmitKeys(vec![(Key::KEY_SPACE, value)])),
        InEvent::Sync => Some(Action::EmitSync),
        _ => None,
    }
}

fn as_input_events(pairs: &[(Key, i32)]) -> Vec<InputEvent> {
    pairs
        .iter()
        .map(|(k, v)| InputEvent::new(EventType::KEY, k.code(), *v))
        .collect()
}

// --- Imperative Shell (effects at boundaries) ---
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

                    // Convert to pure domain event
                    let in_event = match event.kind() {
                        evdev::InputEventKind::Key(key) => Some(InEvent::Key {
                            key,
                            value: event.value(),
                        }),
                        evdev::InputEventKind::Synchronization(_) => Some(InEvent::Sync),
                        _ => None,
                    };

                    if let Some(action) = in_event.and_then(transform) {
                        match action {
                            Action::EmitKeys(pairs) => {
                                let out = as_input_events(&pairs);
                                kbd.emit(&out)?;
                            }
                            Action::EmitSync => {
                                // Forward the actual sync event from source
                                kbd.emit(&[event])?;
                            }
                        }
                    }
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
