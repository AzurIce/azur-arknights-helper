use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    str::FromStr,
    time::Duration,
};

use log::{info, error};


use super::command::{
    host_service::{self, DeviceLong},
    AdbCommand,
};
// use self::command::AdbCommand;

use super::{
    DeviceInfo, MyError,
};

pub mod command;

#[derive(Debug)]
pub enum ResponseStatus {
    Okay,
    Fail,
}

impl FromStr for ResponseStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OKAY" => Ok(Self::Okay),
            "FAIL" => Ok(Self::Fail),
            _ => Err("unknown response status".to_string()),
        }
    }
}

pub struct Host {
    socket_addr: SocketAddrV4,
    tcp_stream: Option<TcpStream>,
    transported_serial: Option<String>,
}

#[cfg(test)]
mod test {
    use crate::adb::command::local_service::ShellCommand;

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

pub fn connect_default() -> Result<Host, String> {
    connect(Ipv4Addr::new(127, 0, 0, 1), 5037)
}

// to get a host connection
pub fn connect(ip: Ipv4Addr, port: u16) -> Result<Host, String> {
    // TODO: if the daemon is not started first start the daemon
    // TODO: or else just use process, don't use socket
    // TODO: or, separate them?
    let mut host = Host::new(SocketAddrV4::new(ip, port));
    host.connect()?;
    Ok(host)
}

impl Host {
    pub fn new(socket_addr: SocketAddrV4) -> Self {
        Self {
            socket_addr,
            tcp_stream: None,
            transported_serial: None,
        }
    }

    pub fn reconnect(&mut self) -> Result<(), String> {
        self.transported_serial = None;
        self.tcp_stream = Some(self.connect()?);
        Ok(())
    }

    fn connect(&self) -> Result<TcpStream, String> {
        let stream = TcpStream::connect(self.socket_addr).map_err(|err| format!("{:?}", err))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|err| format!("{:?}", err))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|err| format!("{:?}", err))?;
        Ok(stream)
    }

    // get devices
    pub fn devices_long(&mut self) -> Result<Vec<DeviceInfo>, MyError> {
        let response = self
            .execute_command(DeviceLong::new())
            .map_err(|err| MyError::Adb(err.to_string()))?;
        Ok(response)
    }

    fn send_request(&mut self, request: String) -> Result<(), String> {
        info!("sending request: {}", request);
        if let Some(tcp_stream) = self.tcp_stream.as_mut() {
            tcp_stream
                .write_all(format!("{:04x}{}", request.len(), request).as_bytes())
                .map_err(|err| format!("tcp error: {:?}", err))
        } else {
            Err("connection closed".to_string())
        }
    }

    pub fn execute_command<T>(
        &mut self,
        command: impl AdbCommand<Output = T>,
    ) -> Result<T, String> {
        // TODO: maybe reconnect every time is a good choice?
        // TODO: no, for transport
        if self.tcp_stream.is_none() {
            info!("reconnecting to server...");
            self.reconnect()?;
        }

        self.send_request(command.raw_command())?;

        command.handle_response(self)
    }

    fn transport<S: AsRef<str>>(&mut self, serial_number: S) -> Result<(), String> {
        let serial_number = serial_number.as_ref();
        info!("transporting to {}...", serial_number);
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

    pub fn read_exact_bytes(&mut self, len: usize) -> Result<Vec<u8>, String> {
        // info!("reading(len: {})...", len);
        let mut buf = [0; 65536];
        if let Some(tcp_stream) = self.tcp_stream.as_mut() {
            tcp_stream
                .read_exact(&mut buf[..len])
                .map_err(|err| format!("{:?}", err))?;
            Ok(buf[..len].to_vec())
        } else {
            Err("connection closed".to_string())
        }
    }

    pub fn read_exact_string(&mut self, len: usize) -> Result<String, String> {
        let bytes = self.read_exact_bytes(len)?;
        let s = std::str::from_utf8(&bytes).map_err(|err| format!("{:?}", err))?;
        Ok(s.to_string())
    }

    pub fn read_string(&mut self) -> Result<String, String> {
        let bytes = self.read_bytes()?;
        let s = std::str::from_utf8(&bytes).map_err(|err| format!("{:?}", err))?;
        Ok(s.to_string())
    }

    pub fn read_bytes(&mut self) -> Result<Vec<u8>, String> {
        let len = self.read_len()?;
        let bytes = self.read_exact_bytes(len)?;
        Ok(bytes)
    }

    pub fn read_to_end(&mut self) -> Result<Vec<u8>, String> {
        let mut response = Vec::new();
        self.tcp_stream
            .as_mut()
            .and_then(|tcp_stream| tcp_stream.read_to_end(&mut response).ok())
            .ok_or("tcp error".to_string())?;
        Ok(response)
    }

    pub fn read_to_end_string(&mut self) -> Result<String, String> {
        let bytes = self.read_to_end()?;
        let s = std::str::from_utf8(&bytes).map_err(|err| format!("{:?}", err))?;
        Ok(s.to_string())
    }

    pub fn read_len(&mut self) -> Result<usize, String> {
        let len = self.read_exact_string(4)?;
        let len = usize::from_str_radix(&len, 16).unwrap();
        Ok(len)
    }

    pub fn read_response_status(&mut self) -> Result<ResponseStatus, String> {
        let status = self.read_exact_string(4)?;
        let status = ResponseStatus::from_str(&status).unwrap();
        Ok(status)
    }

    pub fn check_response_status(&mut self) -> Result<(), String> {
        info!("checking response_status...");
        let status = self.read_response_status()?;
        if let ResponseStatus::Fail = status {
            let reason = self.read_string()?;
            error!("response status is FAIL, reason: {}", reason);
            return Err(reason);
        }
        error!("response status is OKAY");
        Ok(())
    }
}
