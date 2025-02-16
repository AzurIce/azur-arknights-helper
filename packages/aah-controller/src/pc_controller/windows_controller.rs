use std::time::Duration;

use crate::{adb::MyError, Controller};

pub fn create_pc_controller() -> Result<Box<dyn Controller + Sync + Send>, MyError> {
    println!("PcController connecting in platform: windows");

    Ok(Box::new(WindowsController::new()))
}

struct WindowsController {
    
}

impl WindowsController {
    fn new() -> Self {
        println!("Hello, WindowsController");

        Self {
            
        }
    }
}

#[allow(unused_variables)]
impl Controller for WindowsController {
    fn screen_size(&self) -> (u32, u32) {
        unimplemented!("screen_size")
    }

    fn click(&self, x: u32, y: u32) -> Result<(), MyError> {
        unimplemented!("click")
    }

    /// slope_in and slope_out has no effect on [`AdbController`]
    fn swipe(
        &self,
        start: (u32, u32),
        end: (i32, i32),
        duration: Duration,
        slope_in: f32,
        slope_out: f32,
    ) -> Result<(), MyError> {
        unimplemented!("swipe")
    }
    fn raw_screencap(&self) -> Result<Vec<u8>, MyError> {
        unimplemented!("")
    }
    fn screencap(&self) -> Result<image::DynamicImage, MyError> {
        unimplemented!("")
    }

    fn press_home(&self) -> Result<(), MyError> {
        unimplemented!("")
    }

    fn press_esc(&self) -> Result<(), MyError> {
        unimplemented!("")
    }
}