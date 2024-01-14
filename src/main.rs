use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex, mpsc::channel};
use std::time::Duration;
use std::thread;

use hidapi::{HidApi, HidDevice};
use rusqlite::{params, Connection};
use tokio::{task, runtime::Runtime, sync::Mutex as TokioMutex};

const DB_PATH: &str = "keystrokes.db";

fn monitor_device(device: udev::Device) -> Result<(), Box<dyn Error>> {
    //println!("monitoring device...");

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api = Arc::new(HidApi::new()?);
    let conn = Arc::new(Mutex::new(rusqlite::Connection::open(DB_PATH)?));
    let devices: HashMap<String, String> = HashMap::new();
    let (tx, rx) = channel::<i32>();

    //let t1 = thread::spawn(move || {
    //    for received in rx {
    //        println!("received: {:#?}", received);
    //    }
    //});

    let mut enumerator = udev::Enumerator::new().unwrap();
    enumerator.match_property("ID_VENDOR_ID", "045e").unwrap();
    enumerator.match_property("ID_USB_MODEL_ID", "028e").unwrap();
    enumerator.match_attribute("power/control", "on").unwrap();

    let mut i = 0;
    for device in enumerator.scan_devices().unwrap() {
        monitor_device(device);
    }

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

async fn handle_device_event(api: Arc<HidApi>, devices: &mut HashMap<String, Arc<TokioMutex<HidDevice>>>, conn: Arc<Mutex<Connection>>) {
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
