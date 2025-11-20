// #![feature(associated_type_defaults)]
// #![feature(path_file_prefix)]

use std::sync::Arc;

use aah::{AahCore, resource::AahResource};
use auto_play::resource::GitRepoResource;
use clap::{CommandFactory, Parser, Subcommand};
use tracing_subscriber::EnvFilter;

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
    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_env_filter(
            EnvFilter::default()
                .add_directive("aah=info".parse().unwrap())
                .add_directive("aah_core=info".parse().unwrap())
                .add_directive("aah_resource=info".parse().unwrap())
                .add_directive("ap_adb=info".parse().unwrap())
                .add_directive("ap_controller=info".parse().unwrap())
                .add_directive("ap_cv=info".parse().unwrap()),
        )
        .init();
    let cli = Cli::parse();

    let serial = cli.serial_number.unwrap_or("127.0.0.1:16384".to_string());
    if cli.task.is_none() {
        Cli::command().print_help().unwrap();
        return;
    }

    let command = cli.task.as_ref().unwrap();
    let resource = GitRepoResource::<AahResource>::try_load_or_init(
        "./.aah/resources",
        "https://github.com/AzurIce/aah-resources",
        None,
    )
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
