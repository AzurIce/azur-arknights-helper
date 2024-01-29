use crate::adb::{DeviceInfo, utils::read_payload_to_string, AdbTcpStream};

use super::AdbCommand;

#[cfg(test)]
mod test {
    use crate::adb::host;

    use super::*;

    #[test]
    fn test_version() {
        let mut host = host::connect_default().unwrap();
        let res = host.execute_command(Version::new());
        println!("{:?}", res);
    }

    #[test]
    fn test_device_long() {
        let mut host = host::connect_default().unwrap();
        let res = host.execute_command(DeviceLong::new());
        println!("{:?}", res)
    }
}

/// host:version
pub struct Version;

impl Version {
    pub fn new() -> Self {
        Self
    }
}

impl AdbCommand for Version {
    type Output = String;

    fn raw_command(&self) -> String {
        "host:version".to_string()
    }

    fn handle_response(&self, stream: &mut AdbTcpStream) -> Result<Self::Output, String> {
        stream.check_response_status()?;
        read_payload_to_string(stream)
    }
}

/// host:devices-l
pub struct DeviceLong;

impl DeviceLong {
    pub fn new() -> Self {
        Self
    }
}

impl AdbCommand for DeviceLong {
    type Output = Vec<DeviceInfo>;

    fn raw_command(&self) -> String {
        "host:devices-l".to_string()
    }

    fn handle_response(&self, stream: &mut AdbTcpStream) -> Result<Self::Output, String> {
        stream.check_response_status()?;

        let response = read_payload_to_string(stream)?;

        let devices_info = response
            .lines()
            .filter_map(|line| line.try_into().ok())
            .collect::<Vec<DeviceInfo>>();
        return Ok(devices_info);
    }
}

/// host:transport:<serial-number>
pub struct Transport {
    serial_number: String,
}

impl Transport {
    pub fn new(serial_number: String) -> Self {
        Self { serial_number }
    }
}

impl AdbCommand for Transport {
    type Output = ();

    fn raw_command(&self) -> String {
        format!("host:transport:{}", self.serial_number)
    }

    fn handle_response(&self, stream: &mut AdbTcpStream) -> Result<Self::Output, String> {
        stream.check_response_status()
    }
}
