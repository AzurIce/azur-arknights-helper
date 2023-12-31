use std::{path::Path, process::Command};

use forensic_adb::{AndroidStorageInput, DeviceError, Host, UnixPath};
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<(), DeviceError> {
    let maa_lib_dir = Command::new("maa").args(["dir", "library"]).output().expect("failed to execute maa-cli").stdout;
    let maa_lib_dir = String::from_utf8(maa_lib_dir).unwrap();
    println!("{maa_lib_dir}");
    return Ok(());

    let host = Host::default();

    let devices = host.devices::<Vec<_>>().await?;
    println!("Found devices: {:?}", devices);

    let device = host
        .device_or_default(Option::<&String>::None, AndroidStorageInput::default())
        .await?;
    println!("Selected device: {:?}", device);

    let output = device.execute_host_shell_command("id").await?;
    println!("Received response: {:?}", output);

    let output = device.execute_host_shell_command("screencap /storage/emulated/0/_FILES/screen.png").await?;
    println!("screencap output: {:?}", output);

    let mut local_file = File::create("./screen.png").await?;
    let output = device.pull(UnixPath::new("/storage/emulated/0/_FILES/screen.png"), &mut local_file).await?;
    println!("pulled screencap: {:?}", output);

    Ok(())
}