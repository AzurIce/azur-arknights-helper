use std::{net::{TcpStream, ToSocketAddrs}, error::Error, time::Duration};

use forensic_adb::{Host, AndroidStorageInput, Device};
use tokio::task::block_in_place;

pub struct Controller {
    pub inner: Device,
}

impl Default for Controller {
    fn default() -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let device = rt.block_on(async {
            let host = Host::default();

            let devices = host.devices::<Vec<_>>().await.expect("failed to get devices");
            println!("Found devices: {:?}", devices);

            let device = host
                .device_or_default(Option::<&String>::None, AndroidStorageInput::default())
                .await.expect("failed to select device");
            println!("Selected device: {:?}", device);
            device
        });
        Self { inner: device }
    }
}

impl Controller {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn click(&self) {

    }

    pub fn swipe(&self) {

    }

    pub fn screencap(&self) {
        // self.inner.exc
    }
}