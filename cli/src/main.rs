use std::{path::Path, process::Command};

use forensic_adb::{AndroidStorageInput, DeviceError, Host, UnixPath};
use tokio::fs::File;

#[allow(unused)]
mod adb;
mod controller;

use controller::Controller;

#[tokio::main]
async fn main() -> Result<(), DeviceError> {
    // let maa_lib_dir = Command::new("maa").args(["dir", "library"]).output().expect("failed to execute maa-cli").stdout;
    // let maa_lib_dir = String::from_utf8(maa_lib_dir).unwrap();
    // println!("{maa_lib_dir}");
    // return Ok(());

    let device = Controller::new();

    let output = device.inner.execute_host_shell_command("id").await?;
    println!("Received response: {:?}", output);

    let output = device.inner.execute_host_shell_command("screencap /storage/emulated/0/_FILES/screen.png").await?;
    println!("screencap output: {:?}", output);

    let mut local_file = File::create("./screen.png").await?;
    let output = device.inner.pull(UnixPath::new("/storage/emulated/0/_FILES/screen.png"), &mut local_file).await?;
    println!("pulled screencap: {:?}", output);

    Ok(())
}