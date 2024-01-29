use std::{
    io::{self, BufRead, Write},
    process::{ChildStdin, Command, Stdio},
    time::Duration,
};

use crate::adb::{
    command::local_service::{self, ShellCommand},
    connect,
    utils::execute_adb_command,
    AdbTcpStream, Device, MyError,
};
use log::{error, info};

use super::{Controller, Toucher};

#[cfg(test)]
mod test {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
        env::set_current_dir(Path::new("../../../../")).unwrap();
    }

    #[test]
    fn test_controller() {
        init();
        let controller =
            MiniTouchController::connect("127.0.0.1:16384").expect("failed to connect to device");
        controller.screencap().expect("failed to cap the screen");
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
        let mut toucher = MiniToucher::new("127.0.0.1:16384".to_string());
        toucher.click(1000, 1000).unwrap();
    }

    #[test]
    fn test_slowly_swipe() {
        init();
        let mut toucher = MiniToucher::new("127.0.0.1:16384".to_string());
        toucher
            .swipe(
                (2560, 720),
                (-100, 720),
                Duration::from_millis(200),
                2.0,
                0.0,
            )
            .unwrap();
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
    flip_xy: bool,
    max_contact: u32,
    max_x: u32, // 横屏的 x!
    max_y: u32,
    max_pressure: u32,
}

impl MiniToucher {
    pub fn new(serial: String) -> Self {
        Self {
            serial,
            minitouch_stdin: None,
            flip_xy: false,
            max_contact: 0,
            max_x: 0,
            max_y: 0,
            max_pressure: 0,
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
            .args(vec![
                "-s",
                self.serial.as_str(),
                "shell",
                "/data/local/tmp/minitouch",
                "-i",
            ])
            .stdin(Stdio::piped())
            // .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| format!("{:?}", err))?;

        let child_in = minitouch_child
            .stdin
            .take()
            .ok_or("cannot get stdin of minitouch".to_string())?;
        let child_err = minitouch_child
            .stderr
            .take()
            .ok_or("cannot get stdout of minitouch".to_string())?;

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
                    buf = buf
                        .replace("\r\n", "\n")
                        .strip_suffix("\n")
                        .unwrap()
                        .to_string();
                    info!("readed info: {}", buf);
                    if buf.starts_with('^') {
                        let params = &buf.split(' ').skip(1).collect::<Vec<&str>>();
                        let max_contact = u32::from_str_radix(params[0], 10)
                            .map_err(|err| format!("{:?}", err))?;
                        let max_size1 = u32::from_str_radix(params[1], 10)
                            .map_err(|err| format!("{:?}", err))?;
                        let max_size2 = u32::from_str_radix(params[2], 10)
                            .map_err(|err| format!("{:?}", err))?;
                        let max_pressure = u32::from_str_radix(params[3], 10)
                            .map_err(|err| format!("{:?}", err))?;

                        let (max_x, max_y) = if max_size1 > max_size2 {
                            (max_size1, max_size2)
                        } else {
                            self.flip_xy = true;
                            (max_size2, max_size1)
                        };

                        self.max_contact = max_contact;
                        self.max_x = max_x;
                        self.max_y = max_y;
                        self.max_pressure = max_pressure;
                    } else if buf.starts_with('$') {
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
        let (x, y) = if self.flip_xy {
            (y, x)
        } else {
            (x, y)
        };
        self.write_command(format!("d {contact} {x} {y} {pressure}").as_str())
    }

    pub fn mv(&mut self, contact: u32, x: i32, y: i32, pressure: u32) -> Result<(), String> {
        let (x, y) = if self.flip_xy {
            (y, x)
        } else {
            (x, y)
        };
        self.write_command(format!("m {contact} {x} {y} {pressure}").as_str())
    }

    pub fn up(&mut self, contact: u32) -> Result<(), String> {
        self.write_command(format!("u {contact}").as_str())
    }

    pub fn wait(&mut self, duration: Duration) -> Result<(), String> {
        self.write_command(format!("w {}", duration.as_millis()).as_str())
    }
}

const SWIPE_DELAY_MS: u32 = 2;
const CLICK_DELAY_MS: u32 = 50;

impl Toucher for MiniToucher {
    fn click(&mut self, x: u32, y: u32) -> Result<(), String> {
        self.down(0, x, y, 0)?;
        self.commit()?;
        self.wait(Duration::from_millis(CLICK_DELAY_MS as u64))?;
        self.up(0)?;
        self.commit()?;
        Ok(())
    }

    fn swipe(
        &mut self,
        start: (u32, u32),
        end: (i32, i32),
        duration: Duration,
        slope_in: f32,
        slope_out: f32,
    ) -> Result<(), String> {
        self.down(0, start.0, start.1, 0)?;
        self.commit()?;

        // 三次样条插值
        let cubic_spline = |slope_0: f32, slope_1: f32, t: f32| -> f32 {
            let a = slope_0;
            let b = -(2.0 * slope_0 + slope_1 - 3.0);
            let c = -(-slope_0 - slope_1 + 2.0);
            a * t + b * t.powf(2.0) + c * t.powf(3.0)
        };

        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

        for t in (SWIPE_DELAY_MS..duration.as_millis() as u32).step_by(SWIPE_DELAY_MS as usize) {
            let progress =
                cubic_spline(slope_in, slope_out, t as f32 / duration.as_millis() as f32);
            info!("{}", progress);
            let cur_x = lerp(start.0 as f32, end.0 as f32, progress) as i32;
            let cur_y = lerp(start.1 as f32, end.1 as f32, progress) as i32;
            self.mv(0, cur_x as i32, cur_y as i32, 0)?;
            self.commit()?;
            self.wait(Duration::from_millis(SWIPE_DELAY_MS as u64))?;
        }

        self.wait(Duration::from_millis(500))?;
        self.up(0)?;
        self.commit()?;

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
