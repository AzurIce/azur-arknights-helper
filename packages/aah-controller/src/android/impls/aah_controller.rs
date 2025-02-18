use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use color_print::cprintln;

use crate::{
    android::adb::{self},
    android::app::App,
    Toucher,
};

use crate::{android::app::minitouch::MiniTouch, Controller};
use anyhow::{Context, Result};

/// An implementation of [`crate::Controller`]
///
/// This uses minitouch to do the touch events
pub struct AahController {
    pub inner: adb::Device,
    width: u32,
    height: u32,
    // res_dir: PathBuf,
    // minicap: Minicap,
    minitouch: Arc<Mutex<MiniTouch>>,
}

impl AahController {
    pub fn connect(
        device_serial: impl AsRef<str>,
        // res_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        // let res_dir = res_dir.as_ref().to_path_buf();
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

        // let minicap = Minicap::init(&device, &res_dir).map_err(|err| MyError::S(err))?;
        let minitouch = MiniTouch::init(&device).context("minitouch failed to init")?;
        let minitouch = Arc::new(Mutex::new(minitouch));

        let controller = Self {
            inner: device,
            width,
            height,
            // res_dir,
            // minicap,
            minitouch,
        };

        Ok(controller)
    }
}

impl Controller for AahController {
    fn screen_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn click(&self, x: u32, y: u32) -> Result<()> {
        if x > self.width || y > self.height {
            anyhow::bail!("click coord out of screen");
        }
        // cprintln!("<blue>[AahController]</blue>: clicking ({}, {})", x, y);
        // self.inner
        //     .execute_command_by_process(format!("shell input tap {} {}", x, y).as_str())?;
        cprintln!(
            "<blue>[AahController]</blue>: clicking ({}, {}) using minitouch",
            x,
            y
        );
        self.minitouch
            .lock()
            .unwrap()
            .click(x, y)
            .context("minitouch failed to click")?;
        Ok(())
    }

    fn swipe(
        &self,
        start: (u32, u32),
        end: (i32, i32),
        duration: Duration,
        slope_in: f32,
        slope_out: f32,
    ) -> Result<()> {
        cprintln!(
            "<blue>[AahController]</blue>: swiping from {:?} to {:?} for {:?} using minitouch",
            start,
            end,
            duration
        );
        self.minitouch
            .lock()
            .unwrap()
            .swipe(start, end, duration, slope_in, slope_out)
            .unwrap();
        // self.inner.execute_command_by_process(
        //     format!(
        //         "shell input swipe {} {} {} {} {}",
        //         start.0,
        //         start.1,
        //         end.0,
        //         end.1,
        //         duration.as_millis()
        //     )
        //     .as_str(),
        // )?;
        Ok(())
    }
    fn raw_screencap(&self) -> Result<Vec<u8>> {
        self.inner
            .raw_screencap()
            .context("failed to get raw_screencap")
    }
    fn screencap(&self) -> Result<image::DynamicImage> {
        self.inner.screencap().context("failed to get screencap")
        // cprintln!("<blue>[AahController]</blue>: screencapping using minicap...");
        // match self.minicap.get_screen() {
        //     Ok(screen) => Ok(screen),
        //     Err(err) => {
        //         cprintln!("<blue>[AahController]</blue>: failed to get screen through minicap: {err}, use adb instead...");
        //         self.inner.screencap()
        //     }
        // }
    }

    fn press_home(&self) -> Result<()> {
        self.inner
            .execute_command_by_process("shell input keyevent HOME")?;
        Ok(())
    }

    fn press_esc(&self) -> Result<()> {
        self.inner
            .execute_command_by_process("shell input keyevent 111")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{thread::sleep, time::Duration};

    use crate::Controller;

    use super::AahController;

    // #[test]
    // fn test_minicaper() {
    //     let _ = AahController::connect("127.0.0.1:16384", "../../resources").unwrap();
    //     sleep(Duration::from_secs(4));
    // }

    #[test]
    fn test_swipe() {
        let controller = AahController::connect("127.0.0.1:16384").unwrap();
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
