use std::net::{Ipv4Addr, SocketAddrV4};

use log::{info, trace};

use super::{
    command::{
        host_service::{self, DeviceLong},
        AdbCommand,
    },
    AdbTcpStream,
};
// use self::command::AdbCommand;

use super::{DeviceInfo, MyError};

mod command {
    pub const DATA: &[u8; 4] = b"DATA";
    pub const DENT: &[u8; 4] = b"DENT";
    pub const DONE: &[u8; 4] = b"DONE";
    pub const FAIL: &[u8; 4] = b"FAIL";
    pub const LIST: &[u8; 4] = b"LIST";
    pub const OKAY: &[u8; 4] = b"OKAY";
    pub const QUIT: &[u8; 4] = b"QUIT";
    pub const RECV: &[u8; 4] = b"RECV";
    pub const SEND: &[u8; 4] = b"SEND";
    pub const STAT: &[u8; 4] = b"STAT";
}

pub struct Host {
    socket_addr: SocketAddrV4,
    adb_tcp_stream: Option<AdbTcpStream>,
    transported_serial: Option<String>,
}

pub fn connect_default() -> Result<Host, String> {
    connect(Ipv4Addr::new(127, 0, 0, 1), 5037)
}

// to get a host connection
pub fn connect(ip: Ipv4Addr, port: u16) -> Result<Host, String> {
    // TODO: if the daemon is not started first start the daemon
    // TODO: or else just use process, don't use socket
    // TODO: or, separate them?
    let mut host = Host::new(SocketAddrV4::new(ip, port));
    host.reconnect()?;
    Ok(host)
}

impl Host {
    pub fn new(socket_addr: SocketAddrV4) -> Self {
        Self {
            socket_addr,
            adb_tcp_stream: None,
            transported_serial: None,
        }
    }

    pub fn reconnect(&mut self) -> Result<(), String> {
        self.transported_serial = None;
        self.adb_tcp_stream = AdbTcpStream::connect(self.socket_addr).ok();
        Ok(())
    }

    // get devices
    pub fn devices_long(&mut self) -> Result<Vec<DeviceInfo>, MyError> {
        let response = self
            .execute_command(DeviceLong::new())
            .map_err(|err| MyError::AdbCommandError(err.to_string()))?;
        Ok(response)
    }

    pub fn execute_command<T>(
        &mut self,
        command: impl AdbCommand<Output = T>,
    ) -> Result<T, String> {
        // TODO: maybe reconnect every time is a good choice?
        // TODO: no, for transport
        if self.adb_tcp_stream.is_none() {
            trace!("reconnecting to server...");
            self.reconnect()?;
        }

        self.adb_tcp_stream
            .as_mut()
            .ok_or("not connected".to_string())
            .and_then(|stream| stream.execute_command(command))
    }

    fn transport<S: AsRef<str>>(&mut self, serial_number: S) -> Result<(), String> {
        let serial_number = serial_number.as_ref();
        trace!("transporting to {}...", serial_number);
        if let Some(serial) = &self.transported_serial {
            if serial == serial_number {
                info!("already transported, skipped");
                return Ok(());
            }
        }

        self.reconnect()?;

        self.execute_command(host_service::Transport::new(serial_number.to_string()))?;
        self.transported_serial = Some(serial_number.to_string());
        Ok(())
    }

    pub fn execute_local_command<T, S: AsRef<str>>(
        &mut self,
        serial_number: S,
        command: impl AdbCommand<Output = T>,
    ) -> Result<T, String> {
        let serial_number = serial_number.as_ref();
        self.reconnect()?;
        self.transport(serial_number)?;
        self.execute_command(command)
    }
}

#[cfg(test)]
mod test {
    use crate::android::adb::command::local_service::ShellCommand;

    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_host_devices() -> Result<(), MyError> {
        init();
        let mut host = connect_default().unwrap();

        for device_info in host.devices_long()? {
            println!("{:?}", device_info)
        }

        Ok(())
    }

    #[test]
    fn test_shell_command() {
        init();
        let mut host = connect_default().unwrap();

        host.execute_local_command(
            "127.0.0.1:16384".to_string(),
            ShellCommand::new("input swipe 100 100 1000 1000".to_string()),
        )
        .unwrap();
    }
}
