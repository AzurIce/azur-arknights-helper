use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(target_os = "windows"))] {
        mod windows_controller;
        use windows_controller as imp;

    } else if #[cfg(target_os = "macos")] {
        mod macos_controller;
        use macos_controller as imp;

    } else if #[cfg(target_os = "linux")] {
        mod linux_controller;
        use linux_controller as imp;

    } else {
        static PLATFORM_IS_SUPPORTED: bool = false;
        compile_error!("Unsupported platform: {}. Supported platforms are: windows, macos, linux", std::env::consts::OS);
    }
}

pub use imp::create_pc_controller;
