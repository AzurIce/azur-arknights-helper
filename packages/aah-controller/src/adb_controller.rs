use std::time::Duration;

use color_print::cprintln;

use crate::adb::{self, MyError};

use super::Controller;

/// An implementation of [`crate::Controller`]
///
/// This uses minitouch to do the touch events
pub struct AdbController {
    pub inner: adb::Device,
    width: u32,
    height: u32,
}

impl AdbController {
    pub fn connect(device_serial: impl AsRef<str>) -> Result<Self, MyError> {
        let device_serial = device_serial.as_ref();

        cprintln!("<blue>[AahController]</blue>: connecting to {device_serial}...");
        let device = adb::connect(device_serial)?;
        cprintln!("<blue>[AahController]</blue>: connected");

        let screen = device.screencap()?;
        let width = screen.width();
        let height = screen.height();
        cprintln!(
            "<blue>[AahController]</blue>: device screen: {}x{}",
            screen.width(),
            screen.height()
        );

        let controller = Self {
            inner: device,
            width,
            height,
        };

        Ok(controller)
    }
}

impl Controller for AdbController {
    fn screen_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn click(&self, x: u32, y: u32) -> Result<(), MyError> {
        if x > self.width || y > self.height {
            return Err(MyError::S("coord out of screen".to_string()));
        }
        cprintln!(
            "<blue>[AahController]</blue>: clicking ({}, {}) using adb",
            x,
            y
        );
        self.inner
            .execute_command_by_process(format!("shell input tap {} {}", x, y).as_str())?;
        Ok(())
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
        cprintln!(
            "<blue>[AahController]</blue>: swiping from {:?} to {:?} for {:?} using adb",
            start,
            end,
            duration
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
    fn raw_screencap(&self) -> Result<Vec<u8>, MyError> {
        self.inner.raw_screencap()
    }
    fn screencap(&self) -> Result<image::DynamicImage, MyError> {
        self.inner.screencap()
        // cprintln!("<blue>[AahController]</blue>: screencapping using minicap...");
        // match self.minicap.get_screen() {
        //     Ok(screen) => Ok(screen),
        //     Err(err) => {
        //         cprintln!("<blue>[AahController]</blue>: failed to get screen through minicap: {err}, use adb instead...");
        //         self.inner.screencap()
        //     }
        // }
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

#[cfg(test)]
mod test {
    use std::{thread::sleep, time::Duration};

    use crate::Controller;

    use super::AdbController;

    // #[test]
    // fn test_minicaper() {
    //     let _ = AahController::connect("127.0.0.1:16384", "../../resources").unwrap();
    //     sleep(Duration::from_secs(4));
    // }

    #[test]
    fn test_swipe() {
        let controller = AdbController::connect("127.0.0.1:16384").unwrap();
        controller
            .swipe(
                (640, 360),
                (100, 360),
                Duration::from_secs_f32(0.2),
                2.0,
                0.0,
            )
            .unwrap();
        sleep(Duration::from_secs(10));
    }
}
