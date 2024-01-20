#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex, mpsc::channel};
use std::time::Duration;
use std::thread;
use std::sync::atomic::{AtomicU64, Ordering};

use evdev::Device;
use rusqlite::{params, Connection};
use tokio::{task, runtime::Runtime, sync::Mutex as TokioMutex};

const DB_PATH: &str = "keystrokes.db";

fn is_joystick(device: &evdev::Device) -> bool {
    let input_id = device.input_id();
    let vid = input_id.vendor();
    let pid = input_id.product();

    match (vid, pid) {
        (0x045e, 0x028e) => true,
        (0x0c12, 0x0ef8) => true,
        (0x0f0d, 0x0092) => true,
        _ => false,
    }
}

fn debug_device(device: &Device) {
    println!("[Device {:?}]", device.name());
    println!("  Unique Name: {:?}", device.unique_name());
    println!("  Phys Path: {:?}", device.physical_path());
    println!("  Driver Version: {:?}\n", device.driver_version());

    println!("[Supported Events]\n{:?}\n", device.supported_events());

    println!("[Supported Keys]");
    for keyset in device.supported_keys() {
        println!("{:?}", keyset);

        for key in keyset.iter() {
            let variant = match key {
                evdev::Key::BTN_C => "btn_c",
                evdev::Key::BTN_Z => "btn_z",
                evdev::Key::BTN_EAST => "btn_east",
                evdev::Key::BTN_NORTH => "btn_north",
                evdev::Key::BTN_SOUTH => "btn_south",
                evdev::Key::BTN_WEST => "btn_west",
                evdev::Key::BTN_SELECT => "btn_select",
                evdev::Key::BTN_MODE => "btn_mode",
                evdev::Key::BTN_START => "btn_start",
                evdev::Key::BTN_THUMBL => "btn_thumbl",
                evdev::Key::BTN_THUMBR => "btn_thumbr",
                evdev::Key::BTN_TL => "btn_tl",
                evdev::Key::BTN_TR => "btn_tr",
                evdev::Key::BTN_TL2 => "btn_tl2",
                evdev::Key::BTN_TR2 => "btn_tr2",
                evdev::Key::BTN_1 => "btn_1",
                evdev::Key::BTN_9 => "btn_9",
                _ => "unsupported"
            };
            println!("{}: {}", variant, key.0);
        }
    }

    println!("");
    println!("[Supported Absolute Axes]");
    for axes_set in device.supported_absolute_axes() {
        for axis in axes_set.iter() {
            println!("{:?}: {}", axis, axis.0);
        }
    }

    println!("");
    println!("[Supported Switches]\n{:?}\n", device.supported_switches());
    println!("[Properties]\n{:?}\n", device.properties());
    println!("[Misc Properties]\n{:?}\n", device.misc_properties());
}

async fn evdev_test() -> Result<(), Box<dyn Error>> {
    use evdev::{Device, Key};
    let mut monitored_paths: Vec<String> = vec![];

    loop {
        let mut devices = evdev::enumerate()
            .map(|t| t.1)
            .filter(|d| is_joystick(d));

        for device in devices {
            match device.physical_path() {
                Some(p) => {
                    let p_str = p.to_string();
                    match monitored_paths.contains(&p_str) {
                        true => continue,
                        false => monitored_paths.push(p_str.clone()),
                    }
                },
                None => continue,
            }

            debug_device(&device);

            task::spawn(async move {
                println!("starting up thread");
                if let Err(err) = monitor_device(device).await {
                    println!("err while monitoring device: {}", err);
                }
            });
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    println!("ending evdev_test()");
    Ok(())
}

async fn monitor_device(device: evdev::Device) -> Result<(), Box<dyn Error>> {
    let mut events = device.into_event_stream()?;
    let mut timestamps: Vec<(String,String)> = vec![];
    let mut keypresses: HashMap<u16, AtomicU64> = HashMap::new();

    loop {
        let ev = events.next_event().await?;
        if ev.value() != 0 {
            continue
        }

        let code = match ev.kind() {
            evdev::InputEventKind::Key(key) => key.0,
            evdev::InputEventKind::AbsAxis(abs_axis) => abs_axis.0,
            _ => continue,
        };

        keypresses
            .entry(code)
            .and_modify(|counter| { counter.fetch_add(1, Ordering::Relaxed); } )
            .or_insert(AtomicU64::new(1));

        println!("keypresses: {:?}", keypresses);
        println!("{:?}", ev);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    match evdev_test().await {
        Err(err) => println!("err: {}", err),
        _ => {}
    }

    Ok(())
}
