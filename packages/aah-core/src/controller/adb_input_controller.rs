use std::time::Duration;

use log::info;

use crate::adb::{command::local_service, connect, Device, MyError};

use super::Controller;

#[cfg(test)]
mod test {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_controller() {
        init();
        let controller =
            AdbInputController::connect("127.0.0.1:16384").expect("failed to connect to device");
        // controller.screencap().expect("failed to cap the screen");
        controller.swipe_screen(Direction::Left).unwrap()
    }
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct AdbInputController {
    pub inner: Device,
    width: u32,
    height: u32,
}

impl AdbInputController {
    pub fn connect<S: AsRef<str>>(device_serial: S) -> Result<Self, MyError> {
        let device = connect(device_serial)?;
        let controller = Self {
            inner: device,
            width: 0,
            height: 0,
        };
        let screen = controller.screencap()?;

        let controller = Self {
            width: screen.width(),
            height: screen.height(),
            ..controller
        };
        Ok(controller)
    }

    pub fn swipe_screen(&self, direction: Direction) -> Result<(), MyError> {
        // https://android.stackexchange.com/questions/26261/documentation-for-adb-shell-getevent-sendevent
        // https://ktnr74.blogspot.com/2013/06/emulating-touchscreen-interaction-with.html
        let delta = match direction {
            Direction::Up => (0, -(self.height as i32)),
            Direction::Down => (0, self.height as i32),
            Direction::Left => (-(self.width as i32), 0),
            Direction::Right => (self.width as i32, 0),
        };
        let start = (self.width / 2, self.height / 2);

        self.inner
            .execute_command_by_socket(local_service::InputSwipe::new(
                start,
                (start.0 as i32 - 9000, start.1 as i32),
                Duration::from_secs(2),
            ))?;
        self.inner
            .execute_command_by_socket(local_service::InputSwipe::new(
                start,
                (start.0 as i32 + 9000, start.1 as i32),
                Duration::from_secs(2),
            ))?;
        // let now = Instant::now();
        // println!("{}", now.elapsed().as_secs_f32());
        // let mut imgs = Vec::new();
        // while now.elapsed().as_secs_f32() <= 2.0 {
        //     imgs.push(self.screencap()?);
        // }
        Ok(())
    }
}

impl Controller for AdbInputController {
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
