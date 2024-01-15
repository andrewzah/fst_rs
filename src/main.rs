use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex, mpsc::channel};
use std::time::Duration;
use std::thread;

use rusb::{Device, UsbContext};
use rusqlite::{params, Connection};
use tokio::{task, runtime::Runtime, sync::Mutex as TokioMutex};

const DB_PATH: &str = "keystrokes.db";

fn monitor_device<T: UsbContext>(device: Device<T>) -> Result<(), Box<dyn Error>> {
    let device_desc = device.device_descriptor()?;
    let config_desc = device.active_config_descriptor()?;

    println!("monitoring -> Bus {:03} Device {:03} ID {:04x}:{:04x}",
        device.bus_number(),
        device.address(),
        device_desc.vendor_id(),
        device_desc.product_id());

    for iface in config_desc.interfaces() {
        println!("{}", iface.number());

        for iface_desc in iface.descriptors() {
            for endpoint_desc in iface_desc.endpoint_descriptors() {
                println!("{:?}", endpoint_desc);
            }
        }
    }

    //let handle = device.open()?;

    //let vendor_id: u16 = device.property_value("ID_VENDOR_ID").ok_or("")?.into_inner().parse::<u16>();
    //let product_id = device.property_value("ID_USB_MODEL_ID").ok_or("")?;

    //println!("vendor_id: {:?}", vendor_id);

    //let mut hid_device = HidApi::new()?.open(vendor_id, product_id)?;

    //loop {
    //    let mut buf = [0u8; 64];
    //    let read_len = hid_device.read(&mut buf)?;
    //    if read_len > 0 {
    //        println!("{:?}", &buf[..read_len]);
    //    }

    //    thread::sleep(Duration::from_millis(5));
    //}
    Ok(())
}

fn is_joystick<T: rusb::UsbContext>(device: &rusb::Device<T>) -> Result<bool, Box<dyn Error>> {
    let descriptor = device.device_descriptor()?;

    let vid: u16 = descriptor.vendor_id();
    let pid: u16 = descriptor.product_id();


    //println!("{} == {} -> {}", vid, 0x045e, vid == 0x045e);
    //println!("{:#?} == {:#?} -> {}", pid, 0x028e, pid == 0x028e);
    //println!("{}, {}", vid, pid);

    match (vid, pid) {
        (0x045e, 0x028e) => Ok(true),
        _ => Ok(false)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //let hid_api = HidApi::new()?;
    //let hid_api = hidapi_rusb::HidApi::new()?;
    //let conn = Arc::new(Mutex::new(rusqlite::Connection::open(DB_PATH)?));
    //let devices: HashMap<String, String> = HashMap::new();
    //let (tx, rx) = channel::<i32>();

    let r_devices = rusb::devices()?;
    let devices = r_devices
        .iter()
        .filter(|device| match is_joystick(device) {
            Ok(true) => true,
            _ => false,
        });

    for device in devices {
        thread::spawn(move || {
            if let Err(err) = monitor_device(device) {
                println!("ERROR: {}", err);
            }
        });
    }

    //for device in hid_api.device_list() {
    //    println!("{:#?}", device);
    //}
    //println!("num devices: {}", hid_api.device_list().count());

    //let mut enumerator = udev::Enumerator::new().unwrap();
    //for device in enumerator.scan_devices().unwrap() {
    //    println!("{:#?}", device);
    //}
    //println!("udev devices: {}", enumerator.scan_devices().unwrap().count());

    //let t1 = thread::spawn(move || {
    //    for received in rx {
    //        println!("received: {:#?}", received);
    //    }
    //});

    //let mut enumerator = udev::Enumerator::new().unwrap();
    //enumerator.match_property("ID_VENDOR_ID", "045e").unwrap();
    //enumerator.match_property("ID_USB_MODEL_ID", "028e").unwrap();
    //enumerator.match_attribute("power/control", "on").unwrap();

    //let mut i = 0;
    //for device in enumerator.scan_devices().unwrap() {
    //    monitor_device(device);
    //}

    //thread::sleep(Duration::from_secs(1));
    //std::process::exit(0);
    //
    // idVendor=045e, idProduct=028e
    //let threads = vec![t1, t2];
    //for thread in threads {
    //    thread.join();
    //}

    Ok(())
}

fn handle_db() -> Result<(), Box<dyn Error>> {
    Ok(())
}

async fn handle_device_event() {
    println!("test");
}


                //i += 1;
                //println!();
                //println!("{:#?}", device);
                //println!("[properties]");
                //for property in device.properties() {

                //    println!("      - {:?} {:?}", property.name(), property.value());
                //}
                //println!("[attributes]");
                //for attribute in device.attributes() {
                //    println!("      - {:?} {:?}", attribute.name(), attribute.value());
                //}
