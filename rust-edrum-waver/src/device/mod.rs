use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::sync::Mutex;
#[derive(Debug, Deserialize, Clone)]
#[allow(unused_variables)]
pub struct DeviceDetail {
    pub name: String,
    pub position: usize,
}

lazy_static! {
    static ref DEVICES: Mutex<Vec<DeviceDetail>> = Mutex::new(Vec::new());
}

pub fn read_devices() -> Vec<DeviceDetail> {
    let mut devices = DEVICES.lock().unwrap();

    if devices.is_empty() {
        let host = cpal::default_host();
        let available_devices = host.output_devices().unwrap().collect::<Vec<_>>();

        for (position, device) in available_devices.iter().enumerate() {
            let detail = DeviceDetail {
                name: device.name().unwrap(),
                position: position,
            };
            devices.push(detail);
        }
    }

    devices.clone()
}

pub fn dump_devices() {
    println!("Available devices:");
    for device in read_devices().iter() {
        println!(
            "Position: {} | Description: {}",
            device.position, device.name
        );
    }
}
