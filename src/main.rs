#![feature(associated_type_defaults)]
#![feature(path_file_prefix)]

use aah_core::controller::Controller;
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
}