#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex, mpsc::channel};
use std::time::Duration;
use std::thread;

use rusb::{Device, UsbContext};
use rusqlite::{params, Connection};
use tokio::{task, runtime::Runtime, sync::Mutex as TokioMutex};

const DB_PATH: &str = "keystrokes.db";

fn find_endpoint(config_desc: rusb::ConfigDescriptor) -> Option<u8> {
    let mut result = None;

    for iface in config_desc.interfaces() {
        for iface_desc in iface.descriptors() {
            println!("iface num: {}", iface_desc.interface_number());
            println!("  iface num_endpoints: {}", iface_desc.num_endpoints());
            println!("  iface num_endpoint_descs: {}", iface_desc.endpoint_descriptors().count());

            for endpoint_desc in iface_desc.endpoint_descriptors() {
                let dir = match endpoint_desc.direction() {
                    rusb::Direction::In => "in",
                    _ => "out",
                };

                let tt = match endpoint_desc.transfer_type() {
                    rusb::TransferType::Control => "control",
                    rusb::TransferType::Isochronous => "isochronous",
                    rusb::TransferType::Bulk => "bulk",
                    rusb::TransferType::Interrupt => "interrupt",
                };

                println!("endpoint: {}, {}", endpoint_desc.number(), endpoint_desc.address());
                println!("  endpoint dir: {}", dir);
                println!("  endpoint max packet size: {}", endpoint_desc.max_packet_size());
                println!("  endpoint transfer type: {}", tt);

                match endpoint_desc.direction() {
                    rusb::Direction::In => result = Some(endpoint_desc.address()),
                    _ => {},
                };
            }
        }
    }

    result
}

fn monitor_device<T: UsbContext>(device: Device<T>) -> Result<(), Box<dyn Error>> {
    let device_desc = device.device_descriptor()?;
    let config_desc = device.active_config_descriptor()?;

    println!("monitoring -> Bus {:03} Device {:03} ID {:04x}:{:04x}",
        device.bus_number(),
        device.address(),
        device_desc.vendor_id(),
        device_desc.product_id());

    let endpoint = match find_endpoint(config_desc) {
        Some(e) => e,
        None => return Err("Unable to find an endpoint to listen on".into()),
    };
    println!("endpoint: {}", &endpoint);

    let mut handle = device.open()?;

    handle.set_auto_detach_kernel_driver(true)?;

    let mut buf = vec![0u8; 64];
    let timeout = Duration::from_secs(2);

    println!("about to read");

    println!("claiming interface");
    handle.claim_interface(0u8)?;

    println!("reading bytes");
    let bytes_read = handle.read_interrupt(endpoint, &mut buf, timeout)?;

    println!("printing buffer");
    println!("{:?}", buf);

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

//println!("{} == {} -> {}", vid, 0x045e, vid == 0x045e);
//println!("{:#?} == {:#?} -> {}", pid, 0x028e, pid == 0x028e);
//println!("{}, {}", vid, pid);
// 045e, 028e -> haute42, generic cp2040 ?
// 0c12, 0ef8 -> junkfood snackbox micro
fn is_joystick<T: rusb::UsbContext>(device: &rusb::Device<T>) -> bool {
    let descriptor = match device.device_descriptor() {
        Ok(d) => d,
        Err(_) => return false,
    };

    let vid: u16 = descriptor.vendor_id();
    let pid: u16 = descriptor.product_id();

    match (vid, pid) {
        (0x045e, 0x028e) => true,
        (0x0c12, 0x0ef8) => true,
        _ => false,
    }
}

fn rusb_test() -> Result<(), Box<dyn Error>> {
    let r_devices = rusb::devices()?;
    let devices = r_devices
        .iter()
        .filter(|device| is_joystick(device));

    for device in devices {
        thread::spawn(move || {
            if let Err(err) = monitor_device(device) {
                println!("ERROR: {}", err);
            }
        });
    }

    Ok(())
}

fn udev_scan_devices() -> Result<(), Box<dyn Error>> {
    let mut enumerator = udev::Enumerator::new()?;
    enumerator.match_property("ID_VENDOR_ID", "045e")?;
    enumerator.match_property("ID_USB_MODEL_ID", "028e")?;
    //enumerator.match_attribute("power/control", "on").unwrap();

    for device in enumerator.scan_devices()? {
        println!("{:#?}", device);
        println!("[properties]");
        for property in device.properties() {

            println!("      - {:?} {:?}", property.name(), property.value());
        }
        println!("[attributes]");
        for attribute in device.attributes() {
            println!("      - {:?} {:?}", attribute.name(), attribute.value());
        }
    }

    println!("udev devices: {}", enumerator.scan_devices()?.count());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //rusb_test();
    //udev_scan_devices();

    Ok(())
}
