use std::{
    env::args,
    error::Error,
    fmt::Display,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpStream, ToSocketAddrs},
    process::Command,
    str::FromStr,
    time::Duration,
};

use log::{error, info};


use super::command::{
    host_service::{self, DeviceLong},
    AdbCommand,
};
// use self::command::AdbCommand;

use super::{
    Device, DeviceInfo, MyError,
};

pub mod command;

const ADB_SERVER_HOST: &str = "127.0.0.1:5037";
const MAX_RESPONSE_LEN: usize = 1024;

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

// utils for messaging with host
fn read_length<T: Read>(source: &mut T) -> Result<usize, Box<dyn Error>> {
    let mut bytes: [u8; 4] = [0; 4];
    source.read_exact(&mut bytes)?;

    let response = std::str::from_utf8(&bytes)?;

    Ok(usize::from_str_radix(response, 16)?)
}

fn encode_message<S: AsRef<str>>(payload: S) -> Result<String, MyError> {
    let payload = payload.as_ref();

    let len = u16::try_from(payload.len())
        .map(|len| format!("{:0>4X}", len))
        .map_err(|err| MyError::EncodeMessageError(err.to_string()))?;
    Ok(format!("{len}{payload}"))
}

impl Host {
    pub fn new(socket_addr: SocketAddrV4) -> Self {
        Self {
            socket_addr,
            tcp_stream: None,
        }
    }

    pub fn reconnect(&mut self) -> Result<(), String> {
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
        if self.tcp_stream.is_none() {
            info!("reconnecting to server...");
            self.reconnect()?;
        }

        self.send_request(command.raw_command())?;

        command.handle_response(self)
    }

    pub fn execute_local_command<T>(
        &mut self,
        serial_number: String,
        command: impl AdbCommand<Output = T>,
    ) -> Result<T, String> {
        if self.tcp_stream.is_none() {
            info!("reconnecting to server...");
            self.reconnect()?;
        }
        self.execute_command(host_service::Transport::new(serial_number))?;
        self.execute_command(command)
    }

    pub fn read_exact_bytes(&mut self, len: usize) -> Result<Vec<u8>, String> {
        info!("reading(len: {})...", len);
        let mut buf = [0; 65536];
        if let Some(tcp_stream) = self.tcp_stream.as_mut() {
            tcp_stream
                .read_exact(&mut buf[..len])
                .map_err(|err| format!("{:?}", err))?;
            // let buf = std::str::from_utf8(&buf[..len]).unwrap();
            info!("readed {} bytes", buf.len());
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
        let status = self.read_response_status()?;
        if let ResponseStatus::Fail = status {
            let reason = self.read_string()?;
            return Err(reason);
        }
        Ok(())
    }

    pub fn execute_raw_command(
        &mut self,
        command: &str,
        has_output: bool,
        has_length: bool,
    ) -> Result<String, Box<dyn Error>> {
        if let Some(tcp_stream) = self.tcp_stream.as_mut() {
            tcp_stream.write_all(encode_message(command)?.as_bytes())?;
            let bytes = self
                .read_response(has_output, has_length)
                .map_err(|err| MyError::ReadResponseError(format!("{:?}", err)))?;
            let response = std::str::from_utf8(&bytes)?;
            Ok(response.to_owned())
        } else {
            // TODO: refactor
            Ok("".to_string())
        }
    }

    fn read_response(
        &mut self,
        has_output: bool,
        has_length: bool,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut bytes: [u8; MAX_RESPONSE_LEN] = [0; MAX_RESPONSE_LEN];

        self.tcp_stream
            .as_mut()
            .and_then(|tcp_stream| tcp_stream.read_exact(&mut bytes[0..4]).ok());

        if !bytes.starts_with(command::OKAY) {
            let len = self
                .tcp_stream
                .as_mut()
                .and_then(|tcp_stream| {
                    read_length(tcp_stream)
                        .ok()
                        .map(|x| x.min(MAX_RESPONSE_LEN))
                })
                .unwrap();

            self.tcp_stream
                .as_mut()
                .and_then(|tcp_stream| tcp_stream.read_exact(&mut bytes[0..len]).ok());

            let message =
                std::str::from_utf8(&bytes[0..len]).map(|s| format!("adb error: {}", s))?;

            return Err(Box::new(MyError::Adb(message)));
        }

        let mut response = Vec::new();

        if has_output {
            println!("reading output");
            self.tcp_stream
                .as_mut()
                .and_then(|tcp_stream| tcp_stream.read_to_end(&mut response).ok());

            if response.starts_with(command::OKAY) {
                // Sometimes the server produces OKAYOKAY.  Sometimes there is a transport OKAY and
                // then the underlying command OKAY.  This is straight from `chromedriver`.
                response = response.split_off(4);
            }

            if response.starts_with(command::FAIL) {
                // The server may even produce OKAYFAIL, which means the underlying
                // command failed. First split-off the `FAIL` and length of the message.
                response = response.split_off(8);

                let message =
                    std::str::from_utf8(&response).map(|s| format!("adb error: {}", s))?;

                return Err(Box::new(MyError::Adb(message)));
            }

            if has_length {
                if response.len() >= 4 {
                    let message = response.split_off(4);

                    let n = read_length(&mut &*response)?;
                    if n != message.len() {
                        // warn!("adb server response contained hexstring len {} but remaining message length is {}", n, message.len());
                    }

                    // trace!(
                    //     "adb server response was {:?}",
                    //     std::str::from_utf8(&message)?
                    // );

                    return Ok(message);
                } else {
                    return Err(Box::new(MyError::Adb(format!(
                        "adb server response did not contain expected hexstring length: {:?}",
                        std::str::from_utf8(&response)?
                    ))));
                }
            }
        }

        Ok(response)
    }
}
