use std::{collections::BTreeMap, error::Error, fmt::Display, path::PathBuf, process::Command, io::Cursor};

use self::host::Host;

#[allow(unused)]
pub mod host;

#[derive(Debug)]
pub enum MyError {
    Adb(String),
    ParseError(String),
    DeviceNotFound(String),
    HostConnectError,
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
    use super::*;

    #[test]
    fn test_connect() -> Result<(), MyError> {
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
    /// The Adb host which is using to access this device
    host: Host,

    /// Adb device serial number
    serial: String,
}

impl Device {
    pub fn new(host: Host, serial: String) -> Self {
        Self { host, serial }
    }

    // pub fn get_screen_size(&self) -> Result<(u32, u32), MyError> {
    //     let screen = self.screencap()?;
    //     Ok((screen.width(), screen.height()))
    // }

    pub fn screencap(&self) -> Result<image::ImageBuffer<image::Rgb<u8>, Vec<u8>>, MyError> {
        let bytes = self.execute_command_by_process("exec-out screencap -p").expect("failed to screencap");
        
        use image::io::Reader as ImageReader;
        let mut reader = ImageReader::new(Cursor::new(bytes));
        reader.set_format(image::ImageFormat::Png);
        let image = reader.decode().map_err(|err| MyError::ImageDecodeError(format!("{:?}", err)))?.into_rgb8();
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

    pub fn execute_command_by_socket(
        &mut self,
        command: &str,
        has_output: bool,
        has_length: bool,
    ) -> Result<String, MyError> {
        let switch_command = format!("host:transport:{}", self.serial);
        self.host
            .execute_command(&switch_command, false, false)
            .map_err(|err| MyError::Adb(err.to_string()))?;

        let response = self
            .host
            .execute_command(command, has_output, has_length)
            .map_err(|err| MyError::ExecuteCommandFailed(format!("{:?}", err)))?;

        Ok(response.replace("\r\n", "\n"))
    }
}
