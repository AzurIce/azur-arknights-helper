use std::time::Duration;

use log::info;

use crate::adb::{self, MyError};

use super::Controller;

pub struct MiniTouchController {
    pub inner: adb::Device,
    width: u32,
    height: u32,
}

impl MiniTouchController {
    pub fn connect<S: AsRef<str>>(device_serial: S) -> Result<Self, MyError> {
        let device_serial = device_serial.as_ref();
        println!("[MiniTouchController]: connecting to {device_serial}...");

        let device = adb::connect(device_serial)?;
        let controller = Self {
            inner: device,
            width: 0,
            height: 0,
        };
        let screen = controller.screencap()?;

        println!("[MiniTouchController]: connected");
        println!(
            "[MiniTouchController]: device screen: {}x{}",
            screen.width(),
            screen.height()
        );

        let controller = Self {
            width: screen.width(),
            height: screen.height(),
            ..controller
        };
        Ok(controller)
    }
}

impl Controller for MiniTouchController {
    fn screen_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn click(&self, x: u32, y: u32) -> Result<(), MyError> {
        if x > self.width || y > self.height {
            return Err(MyError::S("coord out of screen".to_string()));
        }
        info!("[Controller]: clicking ({}, {})", x, y);
        self.inner
            .execute_command_by_process(format!("shell input tap {} {}", x, y).as_str())?;
        Ok(())
    }

    fn swipe(&self, start: (u32, u32), end: (i32, i32), duration: Duration) -> Result<(), MyError> {
        info!(
            "[Controller]: swiping from {:?} to {:?} for {:?}",
            start, end, duration
        );
        self.inner.execute_command_by_process(
            format!(
                "shell input swipe {} {} {} {} {}",
                start.0,
                start.1,
                end.0,
                end.1,
                duration.as_millis()
            )
            .as_str(),
        )?;
        Ok(())
    }
    fn screencap(&self) -> Result<image::DynamicImage, MyError> {
        self.inner.screencap()
    }

    fn press_home(&self) -> Result<(), MyError> {
        self.inner
            .execute_command_by_process("shell input keyevent HOME")?;
        Ok(())
    }

    fn press_esc(&self) -> Result<(), MyError> {
        self.inner
            .execute_command_by_process("shell input keyevent 111")?;
        Ok(())
    }
}
