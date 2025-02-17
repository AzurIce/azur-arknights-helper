// #![feature(associated_type_defaults)]
// #![feature(path_file_prefix)]

use std::sync::Arc;

use aah_core::{
    arknights::{resource::AahResource, AahCore},
    resource::GitRepoResource,
};
use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// The serial number of the target device, default: 127.0.0.1:16384
    #[arg(short, long)]
    serial_number: Option<String>,

    /// The task name want to execute
    #[command(subcommand)]
    task: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// run task
    Task {
        /// task name
        name: String,
    },
    /// run copilot
    Copilot {
        ///copilot name
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let serial = cli.serial_number.unwrap_or("127.0.0.1:16384".to_string());
    if cli.task.is_none() {
        Cli::command().print_help().unwrap();
        return;
    }

    let command = cli.task.as_ref().unwrap();
    let resource = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(GitRepoResource::<AahResource>::try_init(
            "./.aah/resources",
            "https://github.com/AzurIce/aah-resources",
        ))
        .expect("failed to load resource");
    let aah = AahCore::connect(serial, Arc::new(resource.inner))
        .expect("failed to connect to the device");
    match command {
        Commands::Task { name } => {
            if let Err(err) = aah.run_task(name) {
                println!("task failed: {err}")
            }
        }
        Commands::Copilot { name } => {
            if let Err(err) = aah.run_copilot(name) {
                println!("copilot failed: {err}")
            }
        }
    }
}
