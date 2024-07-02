use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use color_print::cprintln;
use log::{error, info};

use crate::{
    adb::{self, MyError},
    controller::{app::App, Toucher},
};

use super::{
    app::{minicap::Minicap, minitouch::MiniTouch},
    Controller,
};

pub struct AahController {
    pub inner: adb::Device,
    width: u32,
    height: u32,
    res_dir: PathBuf,

    minicap: Minicap,
    // minitouch: Arc<Mutex<MiniTouch>>,
}

impl AahController {
    pub fn connect<S: AsRef<str>, P: AsRef<Path>>(
        device_serial: S,
        res_dir: P,
    ) -> Result<Self, MyError> {
        let res_dir = res_dir.as_ref().to_path_buf();
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

        let minicap = Minicap::init(&device, &res_dir).map_err(|err| MyError::S(err))?;
        // let minitouch = MiniTouch::init(&device, &res_dir).map_err(|err| MyError::S(err))?;
        // let minitouch = Arc::new(Mutex::new(minitouch));

        let controller = Self {
            inner: device,
            width,
            height,
            res_dir,
            minicap,
            // minitouch,
        };

        Ok(controller)
    }
}

impl Controller for AahController {
    fn screen_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn click(&self, x: u32, y: u32) -> Result<(), MyError> {
        if x > self.width || y > self.height {
            return Err(MyError::S("coord out of screen".to_string()));
        }
        // cprintln!("<blue>[AahController]</blue>: clicking ({}, {}) using minitouch", x, y);
        cprintln!("<blue>[AahController]</blue>: clicking ({}, {})", x, y);
        // self.minitouch
        //     .lock()
        //     .unwrap()
        //     .click(x, y)
        //     .map_err(|err| MyError::S(err))?;
        self.inner
            .execute_command_by_process(format!("shell input tap {} {}", x, y).as_str())?;
        Ok(())
    }

    fn swipe(&self, start: (u32, u32), end: (i32, i32), duration: Duration) -> Result<(), MyError> {
        cprintln!(
            "<blue>[AahController]</blue>: swiping from {:?} to {:?} for {:?}",
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
        cprintln!("<blue>[AahController]</blue>: screencapping using minicap...");
        match self.minicap.get_screen() {
            Ok(screen) => Ok(screen),
            Err(err) => {
                cprintln!("<blue>[AahController]</blue>: failed to get screen through minicap: {err}, use adb instead...");
                self.inner.screencap()
            }
        }
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
    use std::{io::Read, net::TcpStream, thread::sleep, time::Duration};

    use super::AahController;

    #[test]
    fn test_minicaper() {
        let controller = AahController::connect("127.0.0.1:16384", "../../resources").unwrap();
        sleep(Duration::from_secs(4));
    }

    #[test]
    fn test_connect() {
        println!("connecting to minicap tcp...");
        let mut connection = TcpStream::connect("localhost:1313").unwrap();
        println!("connected");

        let mut q: Vec<u8> = Vec::new();
        let mut header_occurs = false;
        loop {
            let mut buf = [0u8; 1024];
            match connection.read(&mut buf) {
                Err(err) => {
                    println!("{:?}", err);
                }
                Ok(sz) => {
                    if sz == 0 {
                        continue;
                    }
                    q.extend(buf);
                    println!("{:?}", q);

                    if q.len() >= 24 {
                        header_occurs = true;
                        let header_data = q.drain(0..24).collect::<Vec<u8>>();
                        println!("header_data: {:?}", header_data);
                        break;
                    }
                }
            }
        }
    }
}
