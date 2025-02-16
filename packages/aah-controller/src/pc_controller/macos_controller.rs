use crate::{adb::MyError, Controller};

pub fn create_pc_controller() -> Result<Box<dyn Controller + Sync + Send>, MyError> {
    MyError::new("Unsupported platform: MacOS")
}