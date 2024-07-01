use std::{
    io::{self, BufRead, BufReader, Read},
    net::TcpStream,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{ChildStdin, Command, Stdio},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

use image::DynamicImage;
use log::{error, info};

use crate::adb::{self, command::local_service::ShellCommand, utils::execute_adb_command, MyError};

use super::{minitouch::toucher::MiniToucher, Controller};

pub struct AahController {
    pub inner: adb::Device,
    width: u32,
    height: u32,
    res_dir: PathBuf,
    minicap_stdin: Option<ChildStdin>,
    img: Arc<Mutex<Option<DynamicImage>>>,
}

impl AahController {
    pub fn connect<S: AsRef<str>, P: AsRef<Path>>(
        device_serial: S,
        res_dir: P,
    ) -> Result<Self, MyError> {
        let res_dir = res_dir.as_ref().to_path_buf();
        let device_serial = device_serial.as_ref();

        println!("[AahController]: connecting to {device_serial}...");
        let device = adb::connect(device_serial)?;
        println!("[AahController]: connected");

        let mut controller = Self {
            inner: device,
            width: 0,
            height: 0,
            res_dir,
            minicap_stdin: None,
            img: Arc::new(Mutex::new(None)),
        };
        let screen = controller.screencap()?;
        println!(
            "[AahController]: device screen: {}x{}",
            screen.width(),
            screen.height()
        );

        println!("[AahController]: initializing minicap...");
        controller.init_minicap().unwrap();
        println!("[AahController]: initialized...");

        let controller = Self {
            width: screen.width(),
            height: screen.height(),
            ..controller
        };
        Ok(controller)
    }

    fn check_minicap(&mut self) -> Result<(), String> {
        println!("checking minicap...");
        let mut device_adb_stream = self
            .inner
            .connect_adb_tcp_stream()
            .map_err(|err| format!("failed to connect: {err}"))?;
        let res = device_adb_stream
            .execute_command(ShellCommand::new(
                "LD_LIBRARY_PATH=/data/local/tmp /data/local/tmp/minicap -h".to_string(),
            ))
            .map_err(|err| format!("{:?}", err))?;
        println!("{:?}", res);
        if res.starts_with("Usage") {
            Ok(())
        } else {
            Err("exec failed".to_string())
        }
    }

    fn push_minicap(&mut self) -> Result<(), String> {
        let abi = self.inner.get_abi()?;
        println!("{abi}");
        let bin_path = self.res_dir.join("minicap").join(&abi).join("minicap");
        println!("pushing minicap from {:?}...", bin_path);

        let res = execute_adb_command(
            &self.inner.serial(),
            format!("push {} /data/local/tmp", bin_path.to_str().unwrap()).as_str(),
        )
        .map_err(|err| format!("{:?}", err))?;

        info!("{:?}", res);

        let sdk = self.inner.get_sdk()?;
        // minicap-shared/android-$SDK/$ABI/minicap.so
        let lib_path = self
            .res_dir
            .join("minicap-shared")
            .join(format!("android-{sdk}"))
            .join(&abi)
            .join("minicap.so");
        let res = execute_adb_command(
            &self.inner.serial(),
            format!("push {} /data/local/tmp", lib_path.to_str().unwrap()).as_str(),
        );
        info!("{:?}", res);
        Ok(())
    }

    pub fn init_minicap(&mut self) -> Result<(), String> {
        info!("initializing minitouch...");
        // Need to push
        if self.check_minicap().is_err() {
            self.push_minicap()?;
            self.check_minicap()?;
        }


        execute_adb_command(&self.inner.serial(), "shell killall minicap").unwrap();
        sleep(Duration::from_secs_f32(0.5));

        info!("spawning minicap...");
        let mut minicap_child = Command::new("adb")
            .args(vec![
                "-s",
                self.inner.serial().as_str(),
                "shell",
                "LD_LIBRARY_PATH=/data/local/tmp",
                "/data/local/tmp/minicap",
                "-P",
                // "1920x1080@1920x1080/0", // {RealWidth}x{RealHeight}@{VirtualWidth}x{VirtualHeight}/{Orientation}
                "1920x1080@1920x1080/0", // {RealWidth}x{RealHeight}@{VirtualWidth}x{VirtualHeight}/{Orientation}
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| format!("{:?}", err))?;
        sleep(Duration::from_secs_f32(0.5));

        // let child_in = minicap_child
        //     .stdin
        //     .take()
        //     .ok_or("cannot get stdin of minicap".to_string())?;
        // let child_err = minicap_child
        //     .stderr
        //     .take()
        //     .ok_or("cannot get stderr of minicap".to_string())?;
        let child_out = minicap_child
            .stdout
            .take()
            .ok_or("cannot get stdout of minicap".to_string())?;

        thread::spawn(|| {
            let mut reader = BufReader::new(child_out);
            let mut buf = String::new();
            println!("stdout thread started...");
            loop {
                match reader.read_line(&mut buf) {
                    Ok(sz) => {
                        if sz == 0 {
                            continue;
                        }
                        println!("output: {buf}");
                    }
                    Err(err) => {
                        println!("err: {err}");
                        break;
                    }
                }
            }
            println!("exit stdout thread");
        });

        Command::new("adb")
            .args(vec!["forward", "tcp:1313", "localabstract:minicap"])
            .output()
            .expect("failed to forward minicap tcp port");

        println!("connecting to minicap tcp...");
        let mut connection = TcpStream::connect("localhost:1313").unwrap();
        println!("connected");

        let img = self.img.clone();
        thread::spawn(move || {
            println!("tcp thread started...");
            enum State {
                Head,
                ImgLen,
                Img,
            }
            let mut q: Vec<u8> = Vec::new();
            let mut state = State::Head;
            let mut img_len = 0;
            let mut cnt = 0;
            let mut buf = [0u8; 20480];
            loop {
                match connection.read(&mut buf) {
                    Err(err) => {
                        println!("error: {:?}", err);
                    }
                    Ok(sz) => {
                        if sz == 0 {
                            continue;
                        }
                        // println!("readed {} bytes", sz);
                        q.extend(buf[..sz].iter());

                        match state {
                            State::Head => {
                                if q.len() >= 24 {
                                    let header_data = q.drain(0..24).collect::<Vec<u8>>();
                                    let real_width =
                                        u32::from_le_bytes(header_data[6..=9].try_into().unwrap());
                                    let real_height = u32::from_le_bytes(
                                        header_data[10..=13].try_into().unwrap(),
                                    );
                                    let virtual_width = u32::from_le_bytes(
                                        header_data[14..=17].try_into().unwrap(),
                                    );
                                    let virtual_height = u32::from_le_bytes(
                                        header_data[18..=21].try_into().unwrap(),
                                    );
                                    let orientation = header_data[22];
                                    let flag = header_data[23];
                                    // println!("header_data: {:?}", header_data);
                                    println!(
                                        "header: {}x{}@{}x{}/{} {}",
                                        real_width,
                                        real_height,
                                        virtual_width,
                                        virtual_height,
                                        orientation,
                                        flag
                                    );
                                    state = State::ImgLen;
                                }
                            }
                            State::ImgLen => {
                                if q.len() >= 4 {
                                    let len = q.drain(0..4);
                                    let len = len.as_slice();
                                    img_len = u32::from_le_bytes([len[0], len[1], len[2], len[3]])
                                        as usize;
                                    println!("img_len: {}", img_len);
                                    state = State::Img
                                }
                            }
                            State::Img => {
                                if q.len() >= img_len {
                                    let img_data = q.drain(0..img_len);
                                    let img_data = img_data.as_slice();
                                    let decoded = image::load_from_memory(img_data).unwrap();
                                    println!("recieved frame {}", cnt);
                                    decoded.save(format!("./output{cnt}.png")).unwrap();
                                    cnt += 1;
                                    *img.lock().unwrap() = Some(decoded);
                                    state = State::ImgLen;
                                }
                            }
                        }
                    }
                }
            }
            println!("exit tcp thread");
        });

        // read info
        // self.minicap_stdin = Some(child_in);
        info!("minicap initialized");
        Ok(())
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
