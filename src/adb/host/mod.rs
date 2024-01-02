use std::{
    error::Error,
    fmt::Display,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    time::Duration, process::Command, env::args,
};

use super::{Device, DeviceInfo, MyError};

mod command;

const ADB_SERVER_HOST: &str = "127.0.0.1:5037";
const MAX_RESPONSE_LEN: usize = 1024;

pub struct Host(TcpStream);

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_host() -> Result<(), MyError> {
        let mut host = connect_default()?;

        for device_info in host.devices()? {
            println!("{:?}", device_info)
        }

        Ok(())
    }
}

// to get a host connection
// if cannot connect to the host, it will return a ['MyError::HostConnectError']
pub fn connect_default() -> Result<Host, MyError> {
    connect(ADB_SERVER_HOST)
}

pub fn connect<A: ToSocketAddrs>(host: A) -> Result<Host, MyError> {
    // TODO: if the daemon is not started first start the daemon
    // TODO: or else just use process, don't use socket
    // TODO: or, separate them?
    let stream = TcpStream::connect(host).map_err(|err| MyError::HostConnectError(format!("{:?}", err)))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|err| MyError::HostConnectError(format!("{:?}", err)))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|err| MyError::HostConnectError(format!("{:?}", err)))?;
    Ok(Host(stream))
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

    let len = u16::try_from(payload.len()).map(|len| format!("{:0>4X}", len)).map_err(|err| MyError::EncodeMessageError(err.to_string()))?;
    Ok(format!("{len}{payload}"))
}

impl Host {
    // get devices
    pub fn devices(&mut self) -> Result<Vec<DeviceInfo>, MyError> {
        let response = self.execute_command("host:devices-l", true, true).map_err(|err| MyError::Adb(err.to_string()))?;

        let device_info = response
            .lines()
            .filter_map(|line| line.try_into().ok())
            .collect::<Vec<DeviceInfo>>();

        Ok(device_info)
    }

    // execute command
    pub fn execute_command(
        &mut self,
        command: &str,
        has_output: bool,
        has_length: bool,
    ) -> Result<String, Box<dyn Error>> {
        self.0.write_all(encode_message(command)?.as_bytes())?;
        let bytes = self.read_response(has_output, has_length).map_err(|err| MyError::ReadResponseError(format!("{:?}", err)))?;
        let response = std::str::from_utf8(&bytes)?;
        Ok(response.to_owned())
    }

    fn read_response(
        &mut self,
        has_output: bool,
        has_length: bool,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut bytes: [u8; MAX_RESPONSE_LEN] = [0; MAX_RESPONSE_LEN];

        self.0.read_exact(&mut bytes[0..4])?;

        if !bytes.starts_with(command::OKAY) {
            let len = read_length(&mut self.0)?.min(MAX_RESPONSE_LEN);
            self.0.read_exact(&mut bytes[0..len])?;

            let message =
                std::str::from_utf8(&bytes[0..len]).map(|s| format!("adb error: {}", s))?;

            return Err(Box::new(MyError::Adb(message)));
        }

        let mut response = Vec::new();

        if has_output {
            println!("reading output");
            self.0.read_to_end(&mut response)?;

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
