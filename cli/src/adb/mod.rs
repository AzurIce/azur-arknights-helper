use std::{collections::BTreeMap, error::Error, fmt::Display};

use self::host::AdbHost;

#[allow(unused)]
pub mod host;

#[derive(Debug)]
pub enum MyError {
    Adb(String),
    ParseError(String),
    DeviceNotFound(String),
    HostConnectError
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
            Err(MyError::ParseError("failed to parse device info".to_string()))
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_connect() -> Result<(), MyError>{
        let device = connect("127.0.0.1:16384")?;
        Ok(())
    }
}

// connect to a device using serial,
// if connect failed, it will return a ['MyError::DeviceNotFound']
pub fn connect<S: AsRef<str>>(serial: S) -> Result<Device, MyError> {
    let mut host = host::connect_defualt()?;
    let serial = serial.as_ref().to_string();
    let serials = host
        .devices()?
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
    host: AdbHost,
    serial: String,
}

impl Device {
    pub fn new(host: AdbHost, serial: String) -> Self {
        Self { host, serial }
    }
}
