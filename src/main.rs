use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

use hidapi::{HidApi, HidDevice};
use rusqlite::{params, Connection};
use tokio::{runtime::Runtime, sync::Mutex as TokioMutex};

const DB_PATH: &str = "keystrokes.db";

async fn monitor_device(device: Arc<TokioMutex<HidDevice>>, conn: Arc<Mutex<Connection>>) -> Result<(), Box<dyn Error>> {
    Ok(())
}

async fn handle_device_event(api: Arc<HidApi>, devices: &mut HashMap<String, Arc<TokioMutex<HidDevice>>>, conn: Arc<Mutex<Connection>>) {
    println!("test");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api = Arc::new(HidApi::new()?);
    let conn = Arc::new(Mutex::new(rusqlite::Connection::open(DB_PATH)?));
    let devices = HashMap::new();
    let (sender, receiver) = channel(1000);

    api.set_device_callback(async move |device, connected| {
        handle_device_event(api.clone(), &devices, conn.clone(), sender.clone()).await;
    });

    task::spawn(write_keystrokes_to_database(conn, receiver));

    Runtime::new().unwrap().block_on(async {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            // Perform any additional tasks or checks here
        }
    });

    Ok(())
}
