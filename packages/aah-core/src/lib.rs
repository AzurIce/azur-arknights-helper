//! This crate is the core of [AzurIce/AzurArknightsHelper](https://github.com/AzurIce/AzurArknightsHelper)
//!
#![feature(associated_type_defaults)]
#![feature(path_file_prefix)]

use std::{
    fmt::Debug,
    ops::Deref,
    sync::{Arc, Mutex},
};

use controller::{aah_controller::AahController, Controller};
use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;
use task::TaskEvt;
use vision::analyzer::{
    battle::{
        deploy::{DeployAnalyzer, DeployAnalyzerOutput, EXAMPLE_DEPLOY_OPERS},
        BattleAnalyzer, BattleState,
    },
    Analyzer,
};

use crate::task::Runnable;

pub mod adb;
pub mod controller;
pub mod copilot;
pub mod resource;
pub mod task;
pub mod utils;
pub mod vision;

use resource::Resource;

/// AAH 的实例
pub struct AAH {
    /// [`controller`] 承担设备控制相关操作（比如触摸、截图等）
    pub controller: Box<dyn Controller + Sync + Send>,
    resource: Arc<Resource>,
    screen_cache: Mutex<Option<image::DynamicImage>>,
    task_evt_tx: async_channel::Sender<TaskEvt>,
    pub task_evt_rx: async_channel::Receiver<TaskEvt>,
    task_evt_handler: Vec<Box<dyn Fn(TaskEvt) + Send + Sync>>,

    ocr_engine: OcrEngine,

    runtime: tokio::runtime::Runtime,
}

impl Deref for AAH {
    type Target = Box<dyn Controller + Sync + Send>;
    fn deref(&self) -> &Self::Target {
        &self.controller
    }
}

impl Debug for AAH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AAH")
    }
}

impl AAH {
    /// 连接到 `serial` 指定的设备（`serial` 就是 `adb devices` 里的序列号）
    ///
    /// - `serial`: 设备的序列号
    /// - `res_dir`: 资源目录的路径
    /// - `on_task_evt`: 任务事件的回调函数
    pub fn connect(
        serial: impl AsRef<str>,
        resource: Arc<Resource>,
    ) -> Result<Self, anyhow::Error> {
        let controller = Box::new(AahController::connect(serial, &resource.root())?);

        let (task_evt_tx, task_evt_rx) = async_channel::unbounded();
        let ocr_engine = OcrEngine::new(OcrEngineParams {
            detection_model: Some(
                Model::load_file(resource.root.join("models/text-detection.rten"))?,
            ),
            recognition_model: Some(
                Model::load_file(resource.root.join("models/text-recognition.rten"))?,
            ),
            ..Default::default()
        })
        .unwrap();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        Ok(Self {
            resource,
            controller,
            task_evt_tx,
            task_evt_rx,
            task_evt_handler: vec![],
            screen_cache: Mutex::new(None),

            ocr_engine,
            runtime,
        })
    }

    /// 运行名为 `name` 的任务
    ///
    /// - `name`: 任务名称
    pub fn run_task<S: AsRef<str>>(&self, name: S) -> Result<(), String> {
        let name = name.as_ref().to_string();

        let task = self.resource.get_task(name).ok_or("failed to get task")?;

        task.run(self)
    }

    /// 运行名为 `name` 的作业
    ///
    /// - `name`: 作业名称
    pub fn run_copilot<S: AsRef<str>>(&self, name: S) -> Result<(), String> {
        let name = name.as_ref().to_string();

        let copilot = self
            .resource
            .get_copilot(name)
            .ok_or("failed to get copilot")?;

        copilot.run(self)?;

        Ok(())
    }

    pub fn register_task_evt_handler<F: Fn(TaskEvt) + Send + Sync + 'static>(
        &mut self,
        handler: F,
    ) {
        self.task_evt_handler.push(Box::new(handler));
    }

    /// Get screen cache or capture one. This is for internal analyzer use
    fn screen_cache_or_cap(&self) -> Result<image::DynamicImage, String> {
        let mut screen_cache = self.screen_cache.lock().unwrap();
        if screen_cache.is_none() {
            let screen = self
                .controller
                .screencap()
                .map_err(|err| format!("{err}"))?;
            *screen_cache = Some(screen.clone());
        }
        screen_cache
            .as_ref()
            .map(|i| i.clone())
            .ok_or("screen cache is empty".to_string())
    }

    fn screen_cap_and_cache(&self) -> Result<image::DynamicImage, String> {
        let mut screen_cache = self.screen_cache.lock().unwrap();
        let screen = self
            .controller
            .screencap()
            .map_err(|err| format!("{err}"))?;
        *screen_cache = Some(screen);
        screen_cache
            .as_ref()
            .map(|i| i.clone())
            .ok_or("screen cache is empty".to_string())
    }

    /// Capture a screen, and return decoded image
    pub fn get_screen(&mut self) -> Result<image::DynamicImage, String> {
        self.controller.screencap().map_err(|err| format!("{err}"))
    }

    /// Capture a screen, and return raw data in Png format
    pub fn get_raw_screen(&mut self) -> Result<Vec<u8>, String> {
        self.controller
            .raw_screencap()
            .map_err(|err| format!("{err}"))
    }

    /// 重新加载 resources 中的配置
    // pub fn reload_resources(&mut self) -> Result<(), String> {
    //     let task_config = TaskConfig::load(&self.res_dir)
    //         .map_err(|err| format!("task config not found: {err}"))?;
    //     let navigate_config = NavigateConfig::load(&self.res_dir)
    //         .map_err(|err| format!("navigate config not found: {err}"))?;
    //     self.task_config = task_config;
    //     self.navigate_config = navigate_config;
    //     Ok(())
    // }

    /// 截取当前帧的屏幕内容，分析部署卡片，返回 [`DeployAnalyzerOutput`]
    ///
    /// 通过该函数进行的分析只包含 [`EXAMPLE_DEPLOY_OPERS`] 中的干员
    pub fn analyze_deploy(&self) -> Result<DeployAnalyzerOutput, String> {
        // self.default_oper_list.clone() cost 52s
        let mut analyzer = DeployAnalyzer::new(&self.resource.root, EXAMPLE_DEPLOY_OPERS.to_vec());
        analyzer.analyze(self)
    }

    /// 发起事件
    pub(crate) fn emit_task_evt(&self, evt: TaskEvt) {
        self.runtime.block_on(async {
            self.task_evt_tx.send(evt.clone()).await.unwrap();
        });
        // self.task_evt_tx.send(evt.clone()).unwrap();
        for handler in self.task_evt_handler.iter() {
            (handler)(evt.clone());
        }
    }

    /// 启动战斗分析器，直到战斗结束
    ///
    /// 分析信息会通过 [`TaskEvt::BattleAnalyzerRes`] 事件返回，
    ///
    /// 出于性能考虑，目前待部署区只设置了识别 [`EXAMPLE_DEPLOY_OPERS`] 中的干员
    /// TODO: self.default_oper_list.clone() cost 52s
    pub fn start_battle_analyzer(&self) {
        let mut analyzer = BattleAnalyzer::new(&self.resource.root, EXAMPLE_DEPLOY_OPERS.to_vec());
        while analyzer.battle_state != BattleState::Completed {
            let output = analyzer.analyze(self).unwrap();
            self.emit_task_evt(TaskEvt::BattleAnalyzerRes(output));
        }
    }
}
#[cfg(test)]
mod test {
    use std::{
        path::Path,
        sync::{Mutex, OnceLock},
    };

    use resource::LocalResource;

    use super::*;
    use std::sync::Arc;

    use crate::AAH;

    /// An AAH instance using [`LocalResource`], and connected to `127.0.0.1:16384`
    pub fn aah_for_test() -> AAH {
        let resource = LocalResource::load("../../resources").unwrap();
        AAH::connect("127.0.0.1:16384", Arc::new(resource.into())).unwrap()
    }

    #[test]
    fn foo() {
        let resource = Arc::new(LocalResource::load("../../resources").unwrap().into());
        let mut aah = AAH::connect("127.0.0.1:16384", resource).unwrap();
        aah.register_task_evt_handler(|evt| {
            if let TaskEvt::BattleAnalyzerRes(res) = evt {
                println!("{:?}", res);
            }
        });
        aah.start_battle_analyzer()
    }

    #[test]
    fn test_get_tasks() {
        static S: OnceLock<Mutex<Option<AAH>>> = OnceLock::new();
        let _ = &S;
        let resource = Arc::new(LocalResource::load("../../resources").unwrap().into());
        let aah = AAH::connect("127.0.0.1:16384", resource).unwrap();
        println!("{:?}", aah.resource.get_tasks());
    }

    fn save_screenshot<P: AsRef<Path>, S: AsRef<str>>(aah: &mut AAH, path: P, name: S) {
        let path = path.as_ref();
        let name = name.as_ref();

        let target_path = path.join(name);
        println!("saving screenshot to {:?}", target_path);

        // aah.update_screen().unwrap();
        let screen = aah.get_screen().unwrap();
        screen
            .save_with_format(target_path, image::ImageFormat::Png)
            .unwrap();
    }

    #[test]
    fn screenshot() {
        let resource = Arc::new(LocalResource::load("../../resources").unwrap().into());
        let mut aah = AAH::connect("127.0.0.1:16384", resource).unwrap();
        let dir = "../../resources/templates/MUMU-1920x1080";
        // save_screenshot(dir, "start.png");
        // save_screenshot(dir, "wakeup.png");
        // save_screenshot(dir, "notice.png");
        // save_screenshot(dir, "main.png");
        // save_screenshot(dir, "confirm.png");
        save_screenshot(&mut aah, dir, "1-4_deploying_direction.png");
        // let dir = "/Volumes/Data/Dev/AahAI/dataset/1-4/img";
        // for i in 0..10 {
        //     save_screenshot(&mut aah, dir, format!("{i}.png"));
        //     sleep(Duration::from_secs_f32(0.2))
        // }
        // let dir = "../aah-resource/assets";
        // save_screenshot(dir, "LS-6_1.png");
    }
}
