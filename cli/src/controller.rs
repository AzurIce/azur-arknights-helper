use std::{
    error::Error,
    fs,
    io::Cursor,
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

use image::ImageBuffer;

use crate::adb::{connect, Device, MyError};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_controller() {
        let mut controller =
            Controller::connect("127.0.0.1:16384").expect("failed to connect to device");
        controller.screencap();
    }
}

pub struct Controller {
    pub inner: Device,
    width: u32,
    height: u32,
}

impl Controller {
    fn connect<S: AsRef<str>>(device_serial: S) -> Result<Self, MyError> {
        let device = connect(device_serial)?;
        let controller = Self {
            inner: device,
            width: 0,
            height: 0,
        };
        let screen = controller.screencap()?;

        let controller = Self {
            width: screen.width(),
            height: screen.width(),
            ..controller
        };
        Ok(controller)
    }
}

impl Controller {
    pub fn click(&self, p: (u32, u32)) -> Result<(), MyError> {
        self.inner
            .execute_command_by_process(format!("shell input tap {} {}", p.0, p.1).as_str())?;
        Ok(())
    }

    pub fn swipe(
        &self,
        start: (u32, u32),
        end: (u32, u32),
        duration: Duration,
    ) -> Result<(), MyError> {
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

    pub fn screencap(&self) -> Result<image::RgbImage, MyError> {
        self.inner.screencap()
    }

    pub fn back_to_home(&self) -> Result<(), MyError> {
        self.inner
            .execute_command_by_process("shell input keyevent HOME")?;
        Ok(())
    }

    pub fn press_esc(&self) -> Result<(), MyError> {
        self.inner
            .execute_command_by_process("shell input keyevent 111")?;
        Ok(())
    }
}
