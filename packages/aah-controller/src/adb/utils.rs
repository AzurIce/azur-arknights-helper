use std::{
    io::{Read, Write},
    process::Command,
    str::FromStr,
};

pub fn execute_adb_command(serial: &str, command: &str) -> Result<Vec<u8>, String> {
    let mut args = vec!["-s", serial];
    args.extend(command.split_whitespace().collect::<Vec<&str>>());

    let res = Command::new("adb")
        .args(args)
        .output()
        .map_err(|err| format!("{:?}", err))?
        .stdout;
    Ok(res)
}

// Streaming

pub fn read_exact<T: Read>(source: &mut T, len: usize) -> Result<Vec<u8>, String> {
    let mut buf = [0; 65536];
    source
        .read_exact(&mut buf[..len])
        .map_err(|err| format!("{:?}", err))?;
    Ok(buf[..len].to_vec())
}

pub fn read_exact_to_string<T: Read>(source: &mut T, len: usize) -> Result<String, String> {
    let bytes = read_exact(source, len)?;
    let s = std::str::from_utf8(&bytes).map_err(|err| format!("{:?}", err))?;
    Ok(s.to_string())
}

pub fn read_to_end<T: Read>(source: &mut T) -> Result<Vec<u8>, String> {
    let mut response = Vec::new();
    source
        .read_to_end(&mut response)
        .map_err(|err| format!("{:?}", err))?;
    Ok(response)
}

pub fn read_to_end_to_string<T: Read>(source: &mut T) -> Result<String, String> {
    let bytes = read_to_end(source)?;
    let s = std::str::from_utf8(&bytes).map_err(|err| format!("{:?}", err))?;
    Ok(s.to_string())
}

// Following are more utilized things

pub fn read_payload_len<T: Read>(source: &mut T) -> Result<usize, String> {
    let len = read_exact_to_string(source, 4)?;
    let len = usize::from_str_radix(&len, 16).unwrap();
    Ok(len)
}

pub fn read_payload<T: Read>(source: &mut T) -> Result<Vec<u8>, String> {
    let len = read_payload_len(source)?;
    let bytes = read_exact(source, len)?;
    Ok(bytes)
}

pub fn read_payload_to_string<T: Read>(source: &mut T) -> Result<String, String> {
    let bytes = read_payload(source)?;
    let s = std::str::from_utf8(&bytes).map_err(|err| format!("{:?}", err))?;
    Ok(s.to_string())
}

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

pub fn read_response_status<T: Read>(source: &mut T) -> Result<ResponseStatus, String> {
    let status = read_exact_to_string(source, 4)?;
    let status = ResponseStatus::from_str(&status).unwrap();
    Ok(status)
}

pub fn write_request<T: Write>(target: &mut T, request: String) -> Result<(), String> {
    target
        .write_all(format!("{:04x}{}", request.len(), request).as_bytes())
        .map_err(|err| format!("tcp error: {:?}", err))
}
