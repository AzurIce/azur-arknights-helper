// Automatic daily tasks for Gaku Masu (Gakuen Aidoru Masuta 学園アイドルマスター)

use std::process::Command;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::error::Error;
use aah_core::{resource::Resource, AAH};

// MARK: Configuration

#[derive(Debug, Clone)]
struct Config {
    serial: String,
    resource_path: String,

    emulator_path: String,
    wait_time: u64,
}

// MARK: Main

fn main() {

    let config = initialize_config();

    start_emulator(&config);

    let mut aah = connect_to_device(&config).expect("failed to connect to the device");

    aah.controller.click(504, 209).unwrap();

    // start_kuyo(&aah);

    // aah.get_screen().unwrap().save("./screenshot.png").unwrap();

    // end_emulator(&config);
}

// MARK: Kuyo

fn start_kuyo(aah: &AAH) {
    aah.run_task("gakumasu_start_kuyo").expect("failed to start kuyo");
}

// MARK: Initialize Configuration

fn initialize_config() -> Config {
    Config {
        serial: format!("127.0.0.1:5555"), // 5554 is the default port for LeiDian Emulator
        resource_path: "./resources".to_string(),
        emulator_path: r"E:/Programs/leidian/LDPlayerVK/dnplayer.exe".to_string(),
        wait_time: 30,
    }
}

// MARK: Emulator

fn start_emulator(config: &Config) {
    println!("[GakuMasu] Starting emulator...");

    // 启动模拟器
    Command::new(config.emulator_path.clone())
        .spawn()
        .expect("无法启动模拟器");

    println!("[GakuMasu] Emulator started. Waiting for {} seconds...", config.wait_time);
    
    // 等待10秒钟（可以根据需要调整时间）
    thread::sleep(Duration::from_secs(config.wait_time));

    println!("[GakuMasu] {} seconds passed. Emulator is ready.", config.wait_time);
}

fn end_emulator(config: &Config) {
    println!("[GakuMasu] Ending emulator...");

    // 关闭模拟器
    Command::new("taskkill")
        .args(&["/IM", "dnplayer.exe", "/F"])
        .spawn()
        .expect("无法关闭模拟器");

    println!("[GakuMasu] Emulator ended.");
}

// MARK: Connect to Device

fn connect_to_device(config: &Config) -> Result<AAH, Box<dyn Error>> {
    println!("[GakuMasu] Connecting to device...");

    let serial = config.serial.clone();
    let resource = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(Resource::try_init(&config.resource_path))
        .expect("failed to load resource");

    println!("[GakuMasu] Resource loaded.");

    let aah =
        AAH::connect(serial, Arc::new(resource.into())).expect("failed to connect to the device");

    println!("[GakuMasu] Connected to device.");

    Ok(aah)
}
