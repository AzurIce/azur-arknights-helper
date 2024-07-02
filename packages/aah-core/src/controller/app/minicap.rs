use std::{
    io::{BufRead, BufReader, Read},
    net::TcpStream,
    path::Path,
    process::{Command, Stdio},
    sync::{mpsc, Arc, Mutex},
    thread::{self, sleep},
    time::{Duration, Instant},
};

use image::DynamicImage;
use log::info;

use crate::adb::{command::local_service::ShellCommand, utils::execute_adb_command};

use super::App;

#[derive(Default)]
pub struct Minicap {
    screen_cache: Arc<Mutex<Option<DynamicImage>>>,
}

impl Minicap {
    pub fn get_screen(&self) -> Result<DynamicImage, String>{
        self.screen_cache.lock().unwrap().clone().ok_or("no screen".to_string())
    }
}

enum Evt {
    Info {
        real_width: u32,
        real_height: u32,
        virtual_width: u32,
        virtual_height: u32,
        orientation: u8,
        flag: u8,
    },
    // Log(String),
    NewFrame(Vec<u8>),
}

enum Cmd {
    Stop,
}

impl App for Minicap {
    fn check(device: &crate::adb::Device) -> Result<(), String> {
        println!("[Minicap]: checking minicap...");
        let mut device_adb_stream = device
            .connect_adb_tcp_stream()
            .map_err(|err| format!("minicap failed to connect AdbTcpStream: {err}"))?;
        let res = device_adb_stream
            .execute_command(ShellCommand::new(
                "LD_LIBRARY_PATH=/data/local/tmp /data/local/tmp/minicap -h".to_string(),
            ))
            .map_err(|err| format!("minicap test failed: {:?}", err))?;
        if res.starts_with("Usage") {
            Ok(())
        } else {
            Err("minicap exec failed".to_string())
        }
    }

    fn push<P: AsRef<Path>>(device: &crate::adb::Device, res_dir: P) -> Result<(), String> {
        let res_dir = res_dir.as_ref();

        let abi = device.get_abi()?;
        println!("{abi}");
        let bin_path = res_dir.join("minicap").join(&abi).join("minicap");
        println!("pushing minicap from {:?}...", bin_path);

        let res = execute_adb_command(
            &device.serial(),
            format!("push {} /data/local/tmp", bin_path.to_str().unwrap()).as_str(),
        )
        .map_err(|err| format!("minicap push failed: {:?}", err))?;

        info!("{:?}", res);

        let sdk = device.get_sdk()?;
        // minicap-shared/android-$SDK/$ABI/minicap.so
        let lib_path = res_dir
            .join("minicap-shared")
            .join(format!("android-{sdk}"))
            .join(&abi)
            .join("minicap.so");
        let res = execute_adb_command(
            &device.serial(),
            format!("push {} /data/local/tmp", lib_path.to_str().unwrap()).as_str(),
        );
        info!("{:?}", res);
        Ok(())
    }

    fn init<P: AsRef<Path>>(device: &crate::adb::Device, res_dir: P) -> Result<Self, String>
    where
        Self: Sized,
    {
        Self::prepare(device, res_dir)?;

        execute_adb_command(&device.serial(), "shell killall minicap").unwrap();
        sleep(Duration::from_secs_f32(0.5)); // 得 sleep 一会儿

        println!("[Minicap]: spawing minicap...");
        let mut minicap_child = Command::new("adb")
            .args(vec![
                "-s",
                device.serial().as_str(),
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

        let (evt_tx, evt_rx) = mpsc::channel::<Evt>();
        let (cmd_tx, cmd_rx) = mpsc::channel::<Cmd>();
        let screen_cache = Arc::new(Mutex::new(None));

        let _screen_cache = screen_cache.clone();
        let _serial = device.serial();
        thread::spawn(move || {
            let cmd_rx = cmd_rx;
            let evt_rx = evt_rx;
            let screen_cache = _screen_cache;

            let mut reader = BufReader::new(child_out);
            let mut buf = String::new();
            println!("stdout thread started...");
            loop {
                if let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        Cmd::Stop => {
                            execute_adb_command(&_serial, "shell killall minicap")
                                .expect("failed to kill minicap");
                        }
                    }
                }
                if let Ok(evt) = evt_rx.try_recv() {
                    match evt {
                        Evt::Info {
                            real_width,
                            real_height,
                            virtual_width,
                            virtual_height,
                            orientation,
                            flag,
                        } => {
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
                        }
                        Evt::NewFrame(bytes) => {
                            println!("new frame({} bytes), decoding...", bytes.len());
                            let t = Instant::now();
                            let decoded = image::load_from_memory(&bytes).unwrap();
                            *screen_cache.lock().unwrap() = Some(decoded);
                            println!("updated screen_cache, cost{:?}...", t.elapsed());
                        }
                    }
                }
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

        thread::spawn(move || {
            println!("tcp thread started...");
            let evt_tx = evt_tx;

            enum State {
                Head,
                ImgLen,
                Img,
            }
            let mut q: Vec<u8> = Vec::new();
            let mut state = State::Head;
            let mut img_len = 0;
            let mut buf = [0u8; 20480];
            loop {
                match connection.read(&mut buf) {
                    Err(err) => {
                        println!("error: {:?}", err);
                        break;
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
                                    evt_tx
                                        .send(Evt::Info {
                                            real_width,
                                            real_height,
                                            virtual_width,
                                            virtual_height,
                                            orientation,
                                            flag,
                                        })
                                        .unwrap();
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
                                    // let img_data = img_data.as_slice();
                                    evt_tx.send(Evt::NewFrame(img_data.collect())).unwrap();
                                    // let decoded = image::load_from_memory(img_data).unwrap();
                                    // println!("recieved frame {}", cnt);
                                    // decoded.save(format!("./output{cnt}.png")).unwrap();
                                    // cnt += 1;
                                    // *img.lock().unwrap() = Some(decoded);
                                    state = State::ImgLen;
                                }
                            }
                        }
                    }
                }
            }
        });

        // read info
        // self.minicap_stdin = Some(child_in);
        info!("minicap initialized");
        Ok(Minicap { screen_cache })
    }
}
