// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{http, Manager};

use forensic_adb::{AndroidStorageInput, Host, UnixPath};
use tokio::fs::File;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn update_screen() -> Vec<u8> {
    let host = Host::default();

    let devices = host.devices::<Vec<_>>().await.unwrap();
    println!("Found devices: {:?}", devices);

    let device = host
        .device_or_default(Option::<&String>::None, AndroidStorageInput::default())
        .await.unwrap();
    println!("Selected device: {:?}", device);

    let output = device.execute_host_shell_command("id").await.unwrap();
    println!("Received response: {:?}", output);

    let output = device
        .execute_host_shell_command("screencap /storage/emulated/0/_FILES/screen.png")
        .await.unwrap();
    println!("screencap output: {:?}", output);

    let mut local_file = File::create("./screen.png").await.unwrap();
    let output = device
        .pull(
            UnixPath::new("/storage/emulated/0/_FILES/screen.png"),
            &mut local_file,
        )
        .await.unwrap();
    println!("pulled screencap: {:?}", output);
    let data = std::fs::read(&"./screen.png").unwrap();
    data
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                // window.close_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, update_screen])
        .register_uri_scheme_protocol("screen", |_app, request| {
            if let Ok(data) = std::fs::read(&"./screen.png") {
                println!("{:?}", request);
                http::Response::builder().body(data).unwrap()
            } else {
                http::Response::builder()
                    .status(http::StatusCode::BAD_REQUEST)
                    .header(http::header::CONTENT_TYPE, mime::TEXT_PLAIN.essence_str())
                    .body("failed to read file".as_bytes().to_vec())
                    .unwrap()
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
