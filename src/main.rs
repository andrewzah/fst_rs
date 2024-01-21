#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use evdev::{Device, EventType};
use rusqlite::{params, Connection};
use tokio::{task, runtime::Runtime, sync::Mutex as TokioMutex};

const DB_PATH: &str = "fst.db";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conn = Connection::open(DB_PATH)?;
    let (tx, rx) = mpsc::channel();

    handle_db(conn, rx);

    match evdev_test(tx.clone()).await {
        Err(err) => println!("err: {}", err),
        _ => {}
    }

    Ok(())
}

fn handle_db(conn: Connection, rx: mpsc::Receiver<(String, u16, String)>) -> Result<(), Box<dyn Any + Send>> {
    let handle = thread::spawn(move || {
        loop {
            if let Ok(msg) = rx.recv() {
                let (kind, code, timestamp) = msg;
                conn.execute(
                    "INSERT INTO timestamps (kind, code, joystick_id, timestamp) VALUES (?1, ?2, ?3, ?4)",
                    (kind, code, 99, timestamp)
                );
            }
        }
    });

    Ok(())
}


fn is_joystick(device: &evdev::Device) -> bool {
    if let Some(sk) = device.supported_keys() {
        if !sk.contains(evdev::Key::BTN_START) {
            return false;
        }
    } else {
        return false;
    }

    let se = device.supported_events();
    let has_key = se.contains(EventType::KEY);
    let has_abs = se.contains(EventType::ABSOLUTE);
    let has_rel = se.contains(EventType::RELATIVE);

    match (has_key, has_abs, has_rel) {
        (true, true, true) => true,
        (true, true, false) => true,
        (false, _, _) => false,
        _ => false,
    }
}

fn debug_device(device: &Device) {
    println!("[Device {:?}]", device.name());
    println!("  Unique Name: {:?}", device.unique_name());
    println!("  Phys Path: {:?}", device.physical_path());
    println!("  Driver Version: {:?}\n", device.driver_version());

    println!("[Supported Events]\n{:?}\n", device.supported_events());

    if let Some(keyset) = device.supported_keys() {
        println!("[Supported Keys]");
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

        println!("");
    }

    println!("[Supported Absolute Axes]");
    if let Some(abs_axes_set) = device.supported_absolute_axes() {
        for axis in abs_axes_set.iter() {
            println!("{:?}: {}", axis, axis.0);
        }
        println!("");
    }

    println!("");
    println!("[Supported Switches]\n{:?}\n", device.supported_switches());
    println!("[Properties]\n{:?}\n", device.properties());
    println!("[Misc Properties]\n{:?}\n", device.misc_properties());
}

async fn evdev_test(tx: mpsc::Sender<(String, u16, String)>) -> Result<(), Box<dyn Error>> {
    use evdev::{Device, Key};
    //let mut monitored_paths: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let mut monitored_paths: Vec<String> = vec![];

    loop {
        println!("scanning devices!");

        let mut devices = evdev::enumerate()
            .map(|t| t.1)
            .filter(|d| is_joystick(d));

        for device in devices {
            let tx = tx.clone();

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
                let phys_path = device.physical_path().unwrap().to_string();

                let tx = tx.clone();
                if let Err(err) = monitor_device(device, tx).await {
                    println!("err while monitoring device: {}", err);
                }
            });
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    println!("ending evdev_test()");
    Ok(())
}

async fn monitor_device(device: evdev::Device, tx: mpsc::Sender<(String, u16, String)>) -> Result<(), Box<dyn Error>> {
    let mut events = device.into_event_stream()?;
    let mut timestamps: Vec<(String,String)> = vec![];
    let mut keypresses: HashMap<u16, AtomicU64> = HashMap::new();

    loop {
        let ev = events.next_event().await?;
        if ev.value() != 0 {
            continue
        }

        let (kind, code) = match ev.kind() {
            evdev::InputEventKind::Key(key) => {
                println!("key: {:?}", key);
                (String::from("key"), key.0)
            },
            evdev::InputEventKind::AbsAxis(abs_axis) => {
                println!("abs_axis: {:?}", abs_axis);
                (String::from("abs_axis"), abs_axis.0)
            },
            _ => continue,
        };
        println!("{}: {}", kind, code);

        keypresses
            .entry(code)
            .and_modify(|counter| { counter.fetch_add(1, Ordering::Relaxed); } )
            .or_insert(AtomicU64::new(1));

        let formatted_timestamp = format_timestamp(ev.timestamp());
        if let Err(e) = tx.send((kind, code, formatted_timestamp)) {
            println!("err sending: {}", e);
        }
    }
}

fn format_timestamp(ts: std::time::SystemTime) -> String {
    match ts.duration_since(UNIX_EPOCH) {
        Ok(ut) => format!("{}", ut.as_millis()),
        Err(_) => String::from("err")
    }
}
