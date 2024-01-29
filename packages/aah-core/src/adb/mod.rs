use std::{
    collections::BTreeMap, error::Error, fmt::Display, io::Cursor, process::Command, sync::Mutex,
};

use image::{codecs::png::PngDecoder, DynamicImage};

use self::{
    command::AdbCommand,
    host::Host,
};

pub mod command;
pub mod host;

#[derive(Debug)]
pub enum MyError {
    S(String),
    Adb(String),
    ParseError(String),
    DeviceNotFound(String),
    HostConnectError(String),
    ExecuteCommandFailed(String),
    EncodeMessageError(String),
    ReadResponseError(String),
    ImageDecodeError(String),
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for MyError {}

#[derive(Debug)]
pub struct DeviceInfo {
    pub serial: String,
    pub info: BTreeMap<String, String>,
}

impl TryFrom<&str> for DeviceInfo {
    type Error = MyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Turn "serial\tdevice key1:value1 key2:value2 ..." into a `DeviceInfo`.
        let mut pairs = value.split_whitespace();
        let serial = pairs.next();
        let state = pairs.next();
        if let (Some(serial), Some("device")) = (serial, state) {
            let info: BTreeMap<String, String> = pairs
                .filter_map(|pair| {
                    let mut kv = pair.split(':');
                    if let (Some(k), Some(v), None) = (kv.next(), kv.next(), kv.next()) {
                        Some((k.to_owned(), v.to_owned()))
                    } else {
                        None
                    }
                })
                .collect();

            Ok(DeviceInfo {
                serial: serial.to_owned(),
                info,
            })
        } else {
            Err(MyError::ParseError(
                "failed to parse device info".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use super::*;
    use crate::adb::command::local_service;

    #[test]
    fn test_connect() -> Result<(), MyError> {
        let _device = connect("127.0.0.1:16384")?;
        Ok(())
    }

    #[test]
    fn test_screencap() {
        let device = connect("127.0.0.1:16384").unwrap();

        let start = Instant::now();
        let bytes = device
            .execute_command_by_process("exec-out screencap -p")
            .unwrap();
        println!("cost: {}, {}", start.elapsed().as_millis(), bytes.len());

        let start = Instant::now();
        let bytes2 = device
            .execute_command_by_socket( local_service::ScreenCap::new())
            .unwrap();
        println!("cost: {}, {}", start.elapsed().as_millis(), bytes2.len());

        assert_eq!(bytes, bytes2);
    }
}

// connect to a device using serial,
// if connect failed, it will return a ['MyError::DeviceNotFound']
pub fn connect<S: AsRef<str>>(serial: S) -> Result<Device, MyError> {
    let serial = serial.as_ref();

    let _adb_connect = Command::new("adb")
        .args(["connect", serial])
        .output()
        .map_err(|err| MyError::DeviceNotFound(format!("{:?}", err)))?;
    // TODO: check stdout of it to find whether the connect is success or not
    // TODO: or, actually the following code can already check?

    let mut host = host::connect_default().expect("failed to connect to adb server");

    let serial = serial.to_string();
    let serials = host
        .devices_long()?
        .iter()
        .map(|device_info| device_info.serial.clone())
        .collect::<Vec<String>>();

    if !serials.contains(&serial) {
        Err(MyError::DeviceNotFound(serial.clone()))
    } else {
        Ok(Device::new(host, serial))
    }
}

pub struct Device {
    /// The Adb host which is using to access this device
    host: Mutex<Host>,

    /// Adb device serial number
    serial: String,
}

impl Device {
    pub fn new(host: Host, serial: String) -> Self {
        Self {
            host: Mutex::new(host),
            serial,
        }
    }

    // pub fn get_screen_size(&self) -> Result<(u32, u32), MyError> {
    //     let screen = self.screencap()?;
    //     Ok((screen.width(), screen.height()))
    // }

    pub fn screencap(&self) -> Result<image::DynamicImage, MyError> {
        // let bytes = self
        //     .host
        //     .execute_local_command(self.serial.clone(), local_service::ScreenCap::new())
        //     .expect("failed to screencap");
        let bytes = self
            .execute_command_by_process("exec-out screencap -p")
            .expect("failed to screencap");

        let decoder = PngDecoder::new(Cursor::new(bytes))
            .map_err(|err| MyError::ImageDecodeError(format!("{:?}", err)))?;

        let image = DynamicImage::from_decoder(decoder)
            .map_err(|err| MyError::ImageDecodeError(format!("{:?}", err)))?;
        Ok(image)
    }

    pub fn execute_command_by_process(&self, command: &str) -> Result<Vec<u8>, MyError> {
        let mut args = vec!["-s", self.serial.as_str()];
        args.extend(command.split_whitespace().collect::<Vec<&str>>());

        let res = Command::new("adb")
            .args(args)
            .output()
            .map_err(|err| MyError::ExecuteCommandFailed(format!("{:?}", err)))?
            .stdout;
        Ok(res)
    }

    pub fn execute_command_by_socket<T>(
        &self,
        command: impl AdbCommand<Output = T>,
    ) -> Result<T, MyError> {
        self.host
            .lock()
            .unwrap()
            .execute_local_command(self.serial.clone(), command)
            .map_err(|err| MyError::Adb(err.to_string()))
    }
}
