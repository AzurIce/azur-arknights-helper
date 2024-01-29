use std::{
    io::{Write, Read, self, BufRead},
    process::{ChildStdin, Command, Stdio},
    time::Duration,
};

use crate::adb::{
    command::local_service::{self, ShellCommand},
    connect,
    utils::execute_adb_command,
    AdbTcpStream, Device, MyError,
};
use log::{info, error};

use super::{Controller, Toucher};

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
            MiniTouchController::connect("127.0.0.1:16384").expect("failed to connect to device");
        // controller.screencap().expect("failed to cap the screen");
        controller.swipe_screen(Direction::Left).unwrap()
    }

    use std::{env, path::Path};

    #[test]
    fn test() {
        // let s = AdbTcpStream::connect_device("127.0.0.1:16384").unwrap();
        // let s = AdbTcpStream::connect_device("127.0.0.1:16384").unwrap();
        // let s2 = AdbTcpStream::connect_device("127.0.0.1:16384").unwrap();
    }

    #[test]
    fn test_minitoucher() {
        init();
        env::set_current_dir(Path::new("../../../../")).unwrap();
        let mut toucher = MiniToucher::new("127.0.0.1:16384".to_string());
        toucher.click(1000, 1000).unwrap();
    }
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct MiniToucher {
    serial: String,
    minitouch_stdin: Option<ChildStdin>,
}

impl MiniToucher {
    pub fn new(serial: String) -> Self {
        Self {
            serial,
            minitouch_stdin: None,
        }
    }

    fn check_minitouch(&mut self) -> Result<(), String> {
        let mut device_adb_stream = AdbTcpStream::connect_device(&self.serial)?;
        let res = device_adb_stream
            .execute_command(ShellCommand::new(
                "/data/local/tmp/minitouch -h".to_string(),
            ))
            .map_err(|err| format!("{:?}", err))?;
        if res.starts_with("Usage") {
            Ok(())
        } else {
            Err("exec failed".to_string())
        }
    }

    fn push_minitouch(&mut self) -> Result<(), String> {
        let abi = self.get_abi()?;
        let res = execute_adb_command(
            &self.serial,
            format!("push ./resources/minitouch/{abi}/minitouch /data/local/tmp").as_str(),
        )
        .map_err(|err| format!("{:?}", err))?;
        info!("{:?}", res);
        Ok(())
    }

    fn get_abi(&self) -> Result<String, String> {
        let mut device_adb_stream = AdbTcpStream::connect_device(&self.serial)?;
        device_adb_stream
            .execute_command(ShellCommand::new("getprop ro.product.cpu.abi".to_string()))
    }

    pub fn init(&mut self) -> Result<(), String> {
        info!("initializing minitouch...");
        // Need to push
        if self.check_minitouch().is_err() {
            self.push_minitouch()?;
        }
        self.check_minitouch()?;

        info!("spawning minitouch...");
        let mut minitouch_child = Command::new("adb")
            .args(vec!["-s", self.serial.as_str(), "shell", "/data/local/tmp/minitouch", "-i"])
            .stdin(Stdio::piped())
            // .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| format!("{:?}", err))?;

        let child_in = minitouch_child
            .stdin.take()
            .ok_or("cannot get stdin of minitouch".to_string())?;
        let child_err = minitouch_child.stderr.take().ok_or("cannot get stdout of minitouch".to_string())?;

        // read info
        let mut reader = io::BufReader::new(child_err);
        info!("start reading info...");
        loop {
            let mut buf = String::new();
            match reader.read_line(&mut buf) {
                Err(err) => {
                    error!("{}", err)
                }
                Ok(sz) => {
                    if sz == 0 {
                        info!("readed Ok(0)");
                        break;
                    }
                    buf = buf.replace("\r\n", "\n").strip_suffix("\n").unwrap().to_string();
                    info!("readed info: {}", buf);
                    if buf.starts_with('$') {
                        break;
                    }
                    buf.clear();
                }
            }
        }
        self.minitouch_stdin = Some(child_in);
        info!("minitouch initialized");
        Ok(())
    }

    fn write_command(&mut self, command: &str) -> Result<(), String> {
        if self.minitouch_stdin.is_none() {
            self.init()?;
        }

        info!("writing command: {}", command);
        let mut command = command.to_string();
        if !command.ends_with('\n') {
            command.push('\n');
        }
        self.minitouch_stdin
            .as_mut()
            .ok_or("not conneted".to_string())
            .and_then(|s| {
                s.write_all(command.as_bytes())
                    .map_err(|err| format!("{:?}", err))
            })
    }

    pub fn commit(&mut self) -> Result<(), String> {
        self.write_command("c")
    }

    pub fn reset(&mut self) -> Result<(), String> {
        self.write_command("r")
    }

    pub fn down(&mut self, contact: u32, x: u32, y: u32, pressure: u32) -> Result<(), String> {
        self.write_command(format!("d {contact} {x} {y} {pressure}").as_str())
    }

    pub fn mv(&mut self, contact: u32, x: u32, y: u32, pressure: u32) -> Result<(), String> {
        self.write_command(format!("m {contact} {x} {y} {pressure}").as_str())
    }

    pub fn up(&mut self, contact: u32) -> Result<(), String> {
        self.write_command(format!("u {contact}").as_str())
    }

    pub fn wait(&mut self, duration: Duration) -> Result<(), String> {
        self.write_command(format!("w {}", duration.as_millis()).as_str())
    }
}

impl Toucher for MiniToucher {
    fn click(&mut self, x: u32, y: u32) -> Result<(), String> {
        self.down(0, x, y, 0)?;
        self.commit()?;
        self.wait(Duration::from_millis(200))?;
        self.up(0)?;
        self.commit()?;
        Ok(())
    }

    fn swipe(
        &mut self,
        start: (u32, u32),
        end: (i32, i32),
        duration: Duration,
        slope_in: bool,
        slope_out: bool,
    ) -> Result<(), String> {
        Ok(())
    }
}

pub struct MiniTouchController {
    pub inner: Device,
    width: u32,
    height: u32,
}

impl MiniTouchController {
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

impl Controller for MiniTouchController {
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
