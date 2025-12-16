use bluer::{gatt::remote::Characteristic, Device, Result};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::time::Duration;
use tokio::time::sleep;
use uuid::{uuid, Uuid};
extern crate args;

const BATTERY_SERVICE_UUID: Uuid = uuid!("0000180f-0000-1000-8000-00805f9b34fb");
const BATTERY_LEVEL_CHARACTERISTIC_UUID: Uuid = uuid!("00002a19-0000-1000-8000-00805f9b34fb");
const CHARACTERISTIC_USER_DESCRIPTOR_UUID: Uuid = uuid!("00002901-0000-1000-8000-00805f9b34fb");
//const PNP_ID_CHARACTERISTIC_UUID: Uuid = uuid!("00002a50-0000-1000-8000-00805f9b34fb");
//const ZMK_MANUFACTURER_ID: u16 = 0x1d50;

async fn find_battery_levels(device: &Device) -> Result<Option<Vec<Characteristic>>> {
    let mut battery_levels = Vec::new();
    let uuids = device.uuids().await?.unwrap_or_default();
    if uuids.contains(&BATTERY_SERVICE_UUID) {
        sleep(Duration::from_secs(2)).await;
        if !device.is_connected().await? {
            let mut retries = 2;
            loop {
                match device.connect().await {
                    Ok(()) => break,
                    Err(_) if retries > 0 => {
                        retries -= 1;
                    }
                    Err(err) => return Err(err),
                }
            }
        }
        for service in device.services().await? {
            let uuid = service.uuid().await?;
            if uuid == BATTERY_SERVICE_UUID {
                for char in service.characteristics().await? {
                    let uuid = char.uuid().await?;
                    if uuid == BATTERY_LEVEL_CHARACTERISTIC_UUID {
                        battery_levels.push(char);
                    }
                }
            }
        }
    }
    if battery_levels.is_empty() {
        Ok(None)
    } else {
        return Ok(Some(battery_levels));
    }
}

#[derive(Serialize, Deserialize)]
struct Split {
    level: u8,
    is_peripheral: bool,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct WaybarItem {
    text: String,
    tooltip: String,
    class: String,
}

async fn get_splits(battery_levels: &Vec<Characteristic>) -> Result<Vec<Split>> {
    let mut splits: Vec<Split> = Vec::new();
    for level in battery_levels {
        if level.flags().await?.read {
            let descriptors = level.descriptors().await?;
            let mut name = String::from("Central");
            let mut is_peripheral = false;

            for desc in descriptors {
                let descriptor_uuid = desc.uuid().await?;
                if descriptor_uuid == CHARACTERISTIC_USER_DESCRIPTOR_UUID {
                    let desc_data = desc.read().await?;
                    name = String::from_utf8_lossy(&desc_data).into_owned();
                    is_peripheral = true;
                }
            }
            let value = level.read().await?;
            splits.push(Split {
                level: if value.is_empty() { 0 } else { value[0] },
                is_peripheral: is_peripheral,
                name: name,
            });
        }
    }

    Ok(splits)
}

fn parse_mac_string(mac_str: &String) -> std::result::Result<Vec<u8>, String> {
    mac_str
        .split(':') // Split by colon
        .map(|hex_byte| {
            // Parse each hex string into a u8 (base 16)
            u8::from_str_radix(hex_byte, 16)
                .map_err(|e| format!("Failed to parse '{}': {}", hex_byte, e))
        })
        .collect()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let target_address: Option<bluer::Address> = if args.len() >= 2 {
        match parse_mac_string(&args[1]) {
            Ok(a) => {
                let mac_bytes: Vec<u8> = a.iter().take(6).copied().collect::<Vec<u8>>();
                let array_result: std::result::Result<[u8; 6], _> = mac_bytes.try_into();
                array_result.ok().map(bluer::Address::from)
            }

            Err(_) => None,
        }
    } else {
        None
    };
    env_logger::init();
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    {
        for address in adapter.device_addresses().await? {
            let device = adapter.device(address)?;
            if target_address.is_some()
                && device.address() != bluer::Address::from(target_address.unwrap())
            {
                continue;
            }
            let levels = find_battery_levels(&device).await?;
            if levels.is_some() {
                let splits = get_splits(&levels.unwrap()).await?;
                let tooltip = splits
                    .iter()
                    .map(|k| format!("{}: {}%", k.name, k.level))
                    .collect::<Vec<String>>().join("\n");
                let min_level = splits
                    .into_iter()
                    .min_by(|a, b| a.level.partial_cmp(&b.level).unwrap_or(Ordering::Less))
                    .unwrap()
                    .level;
                let item: WaybarItem = WaybarItem {
                    class: String::from("zmk_battery"),
                    text: format!("{}%", min_level),
                    tooltip: tooltip,
                };
                let json_output = serde_json::to_string(&item).unwrap();
                print!("{}\n", json_output); // Print with a newline
            }
            break;
        }
    }

    Ok(())
}
