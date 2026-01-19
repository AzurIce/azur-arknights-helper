use auto_play::Controller;
use auto_play::AutoPlay;

pub mod utils;
pub mod vision;

pub trait CachedScreenCapper {
    fn screen_cache_or_cap(&self) -> anyhow::Result<image::DynamicImage>;
    fn screen_cap_and_cache(&self) -> anyhow::Result<image::DynamicImage>;
}

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

pub mod actions;
pub mod analyzer;
pub mod resource;

pub use actions::{Action, Runnable, Task};
use anyhow::Context;
use anyhow::Result;
use ocrs::{OcrEngine, OcrEngineParams};
use resource::AahResource;
use rten::Model;

#[derive(Debug, Clone)]
pub enum TaskEvt {
    Log(String),
    Error(String),
    // Add other events as needed by GUI
}

pub struct AahCore {
    pub ap: AutoPlay,
    pub resource: Arc<AahResource>,

    ocr_engine: OcrEngine,

    screen_cache: Mutex<Option<image::DynamicImage>>,
    // task_evt_tx: Sender<TaskEvt>, // If we want to support events
}

impl Debug for AahCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AahCore")
    }
}

impl AahCore {
    /// 连接到 `serial` 指定的设备（`serial` 就是 `adb devices` 里的序列号）
    ///
    /// - `serial`: 设备的序列号
    /// - `res_dir`: 资源目录的路径
    pub fn connect(
        serial: impl AsRef<str>,
        resource: Arc<AahResource>,
    ) -> Result<Self, anyhow::Error> {
        let ap = AutoPlay::connect(serial)?;

        Self::new(ap, resource)
    }

    fn new(ap: AutoPlay, resource: Arc<AahResource>) -> Result<Self, anyhow::Error> {
        let ocr_engine = OcrEngine::new(OcrEngineParams {
            detection_model: Some(
                Model::load_file(resource.root.join("models/text-detection.rten"))
                    .context("cannot load models/text-detection.rten")?,
            ),
            recognition_model: Some(
                Model::load_file(resource.root.join("models/text-recognition.rten"))
                    .context("cannot load models/text-recognition.rten")?,
            ),
            ..Default::default()
        })
        .unwrap();
        Ok(Self {
            resource,
            ocr_engine,
            ap,
            screen_cache: Mutex::new(None),
        })
    }

    pub fn controller(&self) -> &Controller {
        self.ap.controller()
    }

    pub fn get_task(&self, name: impl AsRef<str>) -> Option<&Task<Action>> {
        self.resource.get_task(name.as_ref())
    }

    pub fn get_template(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<image::DynamicImage> {
        self.resource.get_template(path)
    }

    /// 运行名为 `name` 的任务
    ///
    /// - `name`: 任务名称
    pub fn run_task<S: AsRef<str>>(&self, name: S) -> anyhow::Result<()> {
        let name = name.as_ref().to_string();

        let task = self
            .resource
            .get_task(&name)
            .ok_or(anyhow::anyhow!("failed to get task"))?;

        task.execute(self)?;

        Ok(())
    }
    /// 运行名为 `name` 的作业
    ///
    /// - `name`: 作业名称
    pub fn run_copilot<S: AsRef<str>>(&self, name: S) -> anyhow::Result<()> {
        let name = name.as_ref().to_string();

        let copilot = self
            .resource
            .get_copilot(&name)
            .ok_or(anyhow::anyhow!("failed to get copilot"))?;

        copilot.execute(self)?;

        Ok(())
    }

    /// Get screen cache or capture one. This is for internal analyzer use
    pub fn screen_cache_or_cap(&self) -> anyhow::Result<image::DynamicImage> {
        let mut screen_cache = self.screen_cache.lock().unwrap();
        if screen_cache.is_none() {
            let screen = self
                .ap
                .screencap()
                .map_err(|err| anyhow::anyhow!("{err}"))?;
            *screen_cache = Some(screen.clone());
        }
        screen_cache
            .as_ref()
            .map(|i| i.clone())
            .ok_or(anyhow::anyhow!("screen cache is empty"))
    }

    pub fn screen_cap_and_cache(&self) -> anyhow::Result<image::DynamicImage> {
        let mut screen_cache = self.screen_cache.lock().unwrap();
        let screen = self
            .ap
            .screencap()
            .map_err(|err| anyhow::anyhow!("{err}"))?;
        *screen_cache = Some(screen);
        screen_cache
            .as_ref()
            .map(|i| i.clone())
            .ok_or(anyhow::anyhow!("screen cache is empty"))
    }
}