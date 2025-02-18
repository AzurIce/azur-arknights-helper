use std::sync::{Arc, Mutex};

pub mod actions;
pub mod analyzer;
pub mod resource;

use aah_controller::{
    android::{AahController, AdbController},
    Controller,
};
pub use actions::ActionSet;
use anyhow::Context;
use anyhow::Result;
use ocrs::{OcrEngine, OcrEngineParams};
use resource::AahResource;
use rten::Model;

use crate::{resource::GetTask, CachedScreenCapper, Core, TaskRecipe};

pub struct Aah {
    pub controller: Box<dyn Controller>,
    pub resource: Arc<AahResource>,

    ocr_engine: OcrEngine,

    screen_cache: Mutex<Option<image::DynamicImage>>,
}

impl Core for Aah {
    type Controller = Box<dyn Controller>;
    type Resource = AahResource;
    fn controller(&self) -> &Self::Controller {
        &self.controller
    }
    fn resource(&self) -> &Self::Resource {
        &self.resource
    }
}

impl Aah {
    /// 连接到 `serial` 指定的设备（`serial` 就是 `adb devices` 里的序列号）
    ///
    /// - `serial`: 设备的序列号
    /// - `res_dir`: 资源目录的路径
    pub fn connect(
        serial: impl AsRef<str>,
        resource: Arc<AahResource>,
    ) -> Result<Self, anyhow::Error> {
        let controller =
            Box::new(AahController::connect(serial).context("failed to connect AahController")?);

        Self::new(controller, resource)
    }

    /// 连接到 `serial` 指定的设备（`serial` 就是 `adb devices` 里的序列号）
    /// 使用 ADB 控制器
    ///
    /// - `serial`: 设备的序列号
    /// - `res_dir`: 资源目录的路径
    pub fn connect_with_adb_controller(
        serial: impl AsRef<str>,
        resource: Arc<AahResource>,
    ) -> Result<Self, anyhow::Error> {
        let controller = Box::new(AdbController::connect(serial)?);

        Self::new(controller, resource)
    }

    fn new(
        controller: Box<dyn Controller + Sync + Send>,
        resource: Arc<AahResource>,
    ) -> Result<Self, anyhow::Error> {
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
            controller,
            screen_cache: Mutex::new(None),
        })
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

        task.run(self)?;

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

        copilot.run(self)?;

        Ok(())
    }

    // pub fn register_task_evt_handler<F: Fn(TaskEvt) + Send + Sync + 'static>(
    //     &mut self,
    //     handler: F,
    // ) {
    //     self.task_evt_handler.push(Box::new(handler));
    // }

    // /// Capture a screen, and return decoded image
    // pub fn get_screen(&mut self) -> Result<image::DynamicImage, String> {
    //     self.controller.screencap().map_err(|err| format!("{err}"))
    // }

    // /// Capture a screen, and return raw data in Png format
    // pub fn get_raw_screen(&mut self) -> Result<Vec<u8>, String> {
    //     self.controller
    //         .raw_screencap()
    //         .map_err(|err| format!("{err}"))
    // }

    // /// 重新加载 resources 中的配置
    // pub fn reload_resources(&mut self) -> Result<(), String> {
    //     let task_config = TaskConfig::load(&self.res_dir)
    //         .map_err(|err| format!("task config not found: {err}"))?;
    //     let navigate_config = NavigateConfig::load(&self.res_dir)
    //         .map_err(|err| format!("navigate config not found: {err}"))?;
    //     self.task_config = task_config;
    //     self.navigate_config = navigate_config;
    //     Ok(())
    // }

    // /// 截取当前帧的屏幕内容，分析部署卡片，返回 [`DeployAnalyzerOutput`]
    // ///
    // /// 通过该函数进行的分析只包含 [`EXAMPLE_DEPLOY_OPERS`] 中的干员
    // pub fn analyze_deploy(&self) -> Result<DeployAnalyzerOutput, String> {
    //     // self.default_oper_list.clone() cost 52s
    //     let mut analyzer = DeployAnalyzer::new(&self.resource.root, EXAMPLE_DEPLOY_OPERS.to_vec());
    //     analyzer.analyze(self)
    // }

    // /// 发起事件
    // pub(crate) fn emit_task_evt(&self, evt: TaskEvt) {
    //     self.runtime.block_on(async {
    //         self.task_evt_tx.send(evt.clone()).await.unwrap();
    //     });
    //     // self.task_evt_tx.send(evt.clone()).unwrap();
    //     for handler in self.task_evt_handler.iter() {
    //         (handler)(evt.clone());
    //     }
    // }

    // /// 启动战斗分析器，直到战斗结束
    // ///
    // /// 分析信息会通过 [`TaskEvt::BattleAnalyzerRes`] 事件返回，
    // ///
    // /// 出于性能考虑，目前待部署区只设置了识别 [`EXAMPLE_DEPLOY_OPERS`] 中的干员
    // /// TODO: self.default_oper_list.clone() cost 52s
    // pub fn start_battle_analyzer(&self) {
    //     let mut analyzer = BattleAnalyzer::new(&self.resource.root, EXAMPLE_DEPLOY_OPERS.to_vec());
    //     while analyzer.battle_state != BattleState::Completed {
    //         let output = analyzer.analyze(self).unwrap();
    //         self.emit_task_evt(TaskEvt::BattleAnalyzerRes(output));
    //     }
    // }
}

impl CachedScreenCapper for Aah {
    /// Get screen cache or capture one. This is for internal analyzer use
    fn screen_cache_or_cap(&self) -> anyhow::Result<image::DynamicImage> {
        let mut screen_cache = self.screen_cache.lock().unwrap();
        if screen_cache.is_none() {
            let screen = self
                .controller
                .screencap()
                .map_err(|err| anyhow::anyhow!("{err}"))?;
            *screen_cache = Some(screen.clone());
        }
        screen_cache
            .as_ref()
            .map(|i| i.clone())
            .ok_or(anyhow::anyhow!("screen cache is empty"))
    }

    fn screen_cap_and_cache(&self) -> anyhow::Result<image::DynamicImage> {
        let mut screen_cache = self.screen_cache.lock().unwrap();
        let screen = self
            .controller
            .screencap()
            .map_err(|err| anyhow::anyhow!("{err}"))?;
        *screen_cache = Some(screen);
        screen_cache
            .as_ref()
            .map(|i| i.clone())
            .ok_or(anyhow::anyhow!("screen cache is empty"))
    }
}

#[cfg(test)]
mod test {
    use crate::resource::Load;

    use super::*;

    #[test]
    fn test_aah() {
        let resource = AahResource::load("aah-resources").unwrap();
        let resource = Arc::new(resource);
        let aah = Aah::connect("127.0.0.1:16384", resource).unwrap();
        aah.run_task("award").unwrap()
    }
}
