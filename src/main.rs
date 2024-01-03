#![feature(associated_type_defaults)]
#![feature(path_file_prefix)]

use std::{path::Path, process::Command};

use tokio::fs::File;

#[allow(unused)]
mod adb;
#[allow(unused)]
mod controller;
#[allow(unused)]
mod vision;
#[allow(unused)]
mod task;

mod config;

use controller::Controller;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// The serial number of the target device
    #[arg(short, long)]
    serial_number: Option<String>,

    /// The task name want to execute
    task: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Some(task) = cli.task {
        let serial = cli.serial_number.unwrap_or("127.0.0.1:16384".to_string());
        let controller = Controller::connect(serial).expect("failed to connect to the device");
        controller.exec_task(task).expect("failed to execute task");
    }
    // let maa_lib_dir = Command::new("maa").args(["dir", "library"]).output().expect("failed to execute maa-cli").stdout;
    // let maa_lib_dir = String::from_utf8(maa_lib_dir).unwrap();
    // println!("{maa_lib_dir}");
    // return Ok(());

    // let device = Controller::new();

    // let output = device.inner.execute_host_shell_command("id").await?;
    // println!("Received response: {:?}", output);

    // let output = device.inner.execute_host_shell_command("screencap /storage/emulated/0/_FILES/screen.png").await?;
    // println!("screencap output: {:?}", output);

    // let mut local_file = File::create("./screen.png").await?;
    // let output = device.inner.pull(UnixPath::new("/storage/emulated/0/_FILES/screen.png"), &mut local_file).await?;
    // println!("pulled screencap: {:?}", output);

}