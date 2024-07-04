pub mod minicap;
pub mod minitouch;

use std::path::Path;

use crate::adb::Device;

pub trait App {
    fn check(device: &Device) -> Result<(), String>;
    fn push<P: AsRef<Path>>(device: &Device, res_dir: P) -> Result<(), String>;
    fn prepare<P: AsRef<Path>>(device: &Device, res_dir: P) -> Result<(), String> {
        if Self::check(device).is_err() {
            Self::push(device, res_dir)?;
            Self::check(device)?;
        }
        Ok(())
    }

    fn init<P: AsRef<Path>>(device: &Device, res_dir: P) -> Result<Self, String>
    where
        Self: Sized;
}
