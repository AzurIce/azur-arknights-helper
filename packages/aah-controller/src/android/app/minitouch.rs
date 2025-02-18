use std::{
    io::{BufRead, Write},
    path::Path,
    process::{ChildStdin, Command, Stdio},
    thread::{self, sleep},
    time::Duration,
};

use anyhow::Context;
use color_print::cprintln;
use log::trace;
use tempfile::{tempfile, NamedTempFile};

use crate::{
    android::adb::{command::local_service::ShellCommand, utils::execute_adb_command, Device},
    Toucher,
};

const MINITOUCH_ARM: &[u8] = include_bytes!("../../../resources/minitouch/armeabi-v7a/minitouch");
const MINITOUCH_ARM_64: &[u8] = include_bytes!("../../../resources/minitouch/arm64-v8a/minitouch");
const MINITOUCH_X86: &[u8] = include_bytes!("../../../resources/minitouch/x86/minitouch");
const MINITOUCH_X86_64: &[u8] = include_bytes!("../../../resources/minitouch/x86_64/minitouch");

use super::App;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

enum Cmd {
    Stop,
}

/// After initialized, hold a child-stdin to write commands to minitouch
/// If disconnected during using, it should be reconstructed
pub struct MiniTouch {
    minitouch_stdin: ChildStdin,
    state: MiniTouchState,

    cmd_tx: async_channel::Sender<Cmd>,
}

impl Drop for MiniTouch {
    fn drop(&mut self) {
        self.cmd_tx.send_blocking(Cmd::Stop).unwrap();
    }
}

#[allow(unused)]
#[derive(Default)]
pub struct MiniTouchState {
    flip_xy: bool,
    max_contact: u32,
    max_x: u32, // 横屏的 x!
    max_y: u32,
    max_pressure: u32,
}

impl App for MiniTouch {
    fn check(device: &Device) -> anyhow::Result<()> {
        let mut device_adb_stream = device
            .connect_adb_tcp_stream()
            .map_err(|err| anyhow::anyhow!("minitouch connect AdbTcpStream failed :{err}"))?;

        let res = device_adb_stream
            .execute_command(ShellCommand::new(
                "/data/local/tmp/minitouch -h".to_string(),
            ))
            .map_err(|err| anyhow::anyhow!("minitouch test failed: {err}"))?;

        cprintln!("<dim>[Minitouch]: test output: {res}</dim>");
        if !res.starts_with("Usage") {
            anyhow::bail!("minitouch exec failed");
        }
        Ok(())
    }

    fn push(device: &Device) -> anyhow::Result<()> {
        let abi = device
            .get_abi()
            .map_err(|err| anyhow::anyhow!("get abi failed: {err}"))?;
        let minitouch_bytes = match abi.as_str() {
            "armeabi-v7a" => MINITOUCH_ARM,
            "arm64-v8a" => MINITOUCH_ARM_64,
            "x86" => MINITOUCH_X86,
            "x86_64" => MINITOUCH_X86_64,
            _ => anyhow::bail!("unsupported abi: {}", abi),
        };
        let mut tmpfile = NamedTempFile::new().context("failed to create tempfile")?;
        tmpfile
            .write_all(minitouch_bytes)
            .context("failed to write minitouch to tempfile")?;

        let res = execute_adb_command(
            &device.serial(),
            format!("push {} /data/local/tmp", tmpfile.path().to_str().unwrap()).as_str(),
        )
        .map_err(|err| anyhow::anyhow!("minitouch push failed: {:?}", err))?;
        println!("{:?}", String::from_utf8(res));
        println!("renaming {:?} to minitouch...", tmpfile.path().file_name());
        let res = execute_adb_command(
            &device.serial(),
            format!(
                "shell mv /data/local/tmp/{} /data/local/tmp/minitouch",
                tmpfile.path().file_name().unwrap().to_str().unwrap()
            )
            .as_str(),
        )
        .map_err(|err| anyhow::anyhow!("minitouch rename failed: {:?}", err))?;
        println!("{:?}", String::from_utf8(res));
        sleep(Duration::from_millis(200));
        let res = execute_adb_command(&device.serial(), "shell chmod +x /data/local/tmp/minitouch")
            .map_err(|err| anyhow::anyhow!("minitouch push failed: {:?}", err))?;
        println!("{:?}", String::from_utf8(res));
        sleep(Duration::from_millis(200));
        Ok(())
    }

    fn init(device: &Device) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Self::prepare(device)?;

        cprintln!("<dim>[Minitouch]: spawning minitouch...</dim>");
        let mut minitouch_child = Command::new("adb")
            .args(vec![
                "-s",
                device.serial().as_str(),
                "shell",
                "/data/local/tmp/minitouch",
                "-i",
            ])
            .stdin(Stdio::piped())
            // .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to spawn minitouch")?;
        sleep(Duration::from_secs_f32(0.5));

        let child_in = minitouch_child
            .stdin
            .take()
            .ok_or(anyhow::anyhow!("cannot get stdin of minitouch"))?;
        let child_out = minitouch_child
            .stderr
            .take()
            .ok_or(anyhow::anyhow!("cannot get stdout of minitouch"))?;

        let (cmd_tx, cmd_rx) = async_channel::unbounded::<Cmd>();

        let mut minitouch_state = MiniTouchState::default();
        // read info
        let mut reader = std::io::BufReader::new(child_out);
        loop {
            let mut buf = String::new();
            match reader.read_line(&mut buf) {
                Err(err) => {
                    cprintln!("<dim>[Minitouch]: read error: {}</dim>", err);
                    anyhow::bail!("failed to read minitouch info: {}", err);
                }
                Ok(sz) => {
                    if sz == 0 {
                        // println!("readed Ok(0)");
                        continue;
                    }
                    buf = buf
                        .replace("\r\n", "\n")
                        .strip_suffix("\n")
                        .unwrap()
                        .to_string();
                    println!("readed info: {}", buf);
                    if buf.starts_with('^') {
                        let params = &buf.split(' ').skip(1).collect::<Vec<&str>>();
                        let max_contact = u32::from_str_radix(params[0], 10).unwrap();
                        let max_size1 = u32::from_str_radix(params[1], 10).unwrap();
                        let max_size2 = u32::from_str_radix(params[2], 10).unwrap();
                        let max_pressure = u32::from_str_radix(params[3], 10).unwrap();

                        let mut flip_xy = false;
                        let (max_x, max_y) = if max_size1 > max_size2 {
                            (max_size1, max_size2)
                        } else {
                            flip_xy = true;
                            (max_size2, max_size1)
                        };

                        minitouch_state = MiniTouchState {
                            flip_xy,
                            max_contact,
                            max_x,
                            max_y,
                            max_pressure,
                        };
                        // minitouch_state.flip_xy = flip_xy;
                        // minitouch_state.max_contact = max_contact;
                        // minitouch_state.max_x = max_x;
                        // minitouch_state.max_y = max_y;
                        // minitouch_state.max_pressure = max_pressure;
                        cprintln!(
                            "<dim>[MiniTouch]: {} {}x{} {} flip: {}</dim>",
                            max_contact,
                            max_x,
                            max_y,
                            max_pressure,
                            flip_xy,
                        );
                    } else if buf.starts_with('$') {
                        break;
                    }
                }
            }
        }

        thread::spawn(move || {
            let cmd_rx = cmd_rx;

            loop {
                thread::sleep(Duration::from_millis(50));
                if let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        Cmd::Stop => break,
                    };
                }
            }
        });
        cprintln!("<dim>[Minitouch]: minitouch initialized</dim>");
        Ok(MiniTouch {
            minitouch_stdin: child_in,
            state: minitouch_state,
            cmd_tx,
        })
    }
}

/// A Toucher based n [MiniTouch](https://github.com/DeviceFarmer/minitouch)
impl MiniTouch {
    fn write_command(&mut self, command: &str) -> anyhow::Result<()> {
        println!("writing command: {:?}", command);
        let mut command = command.to_string();
        if !command.ends_with('\n') {
            command.push('\n');
        }
        self.minitouch_stdin
            .write_all(command.as_bytes())
            .context("failed to write command")
    }

    pub fn commit(&mut self) -> anyhow::Result<()> {
        self.write_command("c")
    }

    pub fn reset(&mut self) -> anyhow::Result<()> {
        self.write_command("r")
    }

    pub fn down(&mut self, contact: u32, x: u32, y: u32, pressure: u32) -> anyhow::Result<()> {
        // On MuMu emulator, the x-y is flipped and the y is also flipped (???)
        let (x, y) = if self.state.flip_xy {
            (self.state.max_y.saturating_add_signed(-(y as i32)), x)
        } else {
            (x, y)
        };
        // let y = self.state.max_y.saturating_add_signed(-(y as i32));
        self.write_command(format!("d {contact} {x} {y} {pressure}").as_str())
    }

    pub fn mv(&mut self, contact: u32, x: i32, y: i32, pressure: u32) -> anyhow::Result<()> {
        // On MuMu emulator, the x-y is flipped and the y is also flipped (???)
        let (x, y) = if self.state.flip_xy {
            (self.state.max_y as i32 - y, x)
        } else {
            (x, y)
        };
        // let y = self.state.max_y as i32 - y;
        self.write_command(format!("m {contact} {x} {y} {pressure}").as_str())
    }

    pub fn up(&mut self, contact: u32) -> anyhow::Result<()> {
        self.write_command(format!("u {contact}").as_str())
    }

    pub fn wait(&mut self, duration: Duration) -> anyhow::Result<()> {
        self.write_command(format!("w {}", duration.as_millis()).as_str())
    }
}

const SWIPE_DELAY_MS: u32 = 5;
const CLICK_DELAY_MS: u32 = 50;

impl Toucher for MiniTouch {
    fn click(&mut self, x: u32, y: u32) -> anyhow::Result<()> {
        self.down(0, x, y, self.state.max_pressure)?;
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
    ) -> anyhow::Result<()> {
        println!("{start:?} {end:?}");
        self.down(0, start.0, start.1, self.state.max_pressure)?;
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
            let progress = progress.min(1.0).max(0.0);
            // info!("{}", progress);
            println!("{progress}");
            let cur_x = lerp(start.0 as f32, end.0 as f32, progress) as i32;
            let cur_y = lerp(start.1 as f32, end.1 as f32, progress) as i32;
            println!("{cur_x} {cur_y}");
            self.mv(0, cur_x as i32, cur_y as i32, self.state.max_pressure)?;
            self.commit()?;
            self.wait(Duration::from_millis(SWIPE_DELAY_MS as u64))?;
            thread::sleep(Duration::from_millis(SWIPE_DELAY_MS as u64));
        }

        // self.mv(0, end.0, end.1, 0)?;
        self.wait(Duration::from_millis(200))?;
        self.commit()?;
        thread::sleep(Duration::from_millis(200));
        self.up(0)?;
        self.commit()?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::android::adb::connect;

    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_minitoucher() {
        init();
        // mumu
        let device = connect("127.0.0.1:16384").unwrap();
        let mut toucher = MiniTouch::init(&device).unwrap();
        // toucher.click(10, 10).unwrap();
        // toucher.click(100, 100).unwrap();
        toucher.click(822, 762).unwrap();
        thread::sleep(Duration::from_secs_f32(2.0));

        // leidian
        let device = connect("emulator-5554").unwrap();
        let mut toucher = MiniTouch::init(&device).unwrap();
        toucher.click(822, 762).unwrap();
        thread::sleep(Duration::from_secs_f32(2.0));
    }

    #[test]
    fn test_slowly_swipe() {
        init();
        // let device = connect("127.0.0.1:16384").unwrap();
        let device = connect("emulator-5554").unwrap();
        let mut toucher = MiniTouch::init(&device).unwrap();
        toucher
            .swipe(
                (1780, 400),
                (400, 400),
                Duration::from_millis(400),
                2.0,
                0.0,
            )
            .unwrap();
        thread::sleep(Duration::from_secs_f32(2.0))
    }
}
