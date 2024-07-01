#![feature(associated_type_defaults)]
#![feature(path_file_prefix)]

use std::{
    error::Error,
    path::{Path, PathBuf},
};

use config::{navigate::NavigateConfig, task::TaskConfig};
use controller::{aah_controller::AahController, minitouch, Controller};
use task::builtins::BuiltinTask;
use vision::analyzer::{
    deploy::{DeployAnalyzer, DeployAnalyzerOutput},
    Analyzer,
};

use crate::task::Task;

pub mod adb;
pub mod config;
pub mod controller;
pub mod task;
pub mod vision;

/// AAH 的实例
pub struct AAH {
    pub res_dir: PathBuf,
    /// [`controller`] 承担设备控制相关操作（比如触摸、截图等）
    pub controller: Box<dyn Controller + Sync + Send>,
    /// 由 `tasks.toml` 和 `tasks` 目录加载的任务配置
    pub task_config: TaskConfig,
    /// 由 `navigates.toml` 加载的导航配置
    pub navigate_config: NavigateConfig,
    /// 屏幕内容的缓存
    pub screen_cache: Option<image::DynamicImage>,
}

impl AAH {
    /// 连接到 `serial` 指定的设备（`serial` 就是 `adb devices` 里的序列号）
    /// - `serial`: 设备的序列号
    /// - `res_dir`: 资源目录的路径
    pub fn connect<S: AsRef<str>, P: AsRef<Path>>(
        serial: S,
        res_dir: P,
    ) -> Result<Self, Box<dyn Error>> {
        let res_dir = res_dir.as_ref().to_path_buf();
        let task_config =
            TaskConfig::load(&res_dir).map_err(|err| format!("task config not found: {err}"))?;
        let navigate_config = NavigateConfig::load(&res_dir)
            .map_err(|err| format!("navigate config not found: {err}"))?;
        // let controller = Box::new(AdbInputController::connect(serial)?);
        let controller = Box::new(AahController::connect(serial, &res_dir)?);
        Ok(Self {
            res_dir,
            controller,
            task_config,
            navigate_config,
            screen_cache: None,
        })
    }

    /// 运行名为 `name` 的任务
    pub fn run_task<S: AsRef<str>>(&self, name: S) -> Result<(), String> {
        let name = name.as_ref().to_string();

        let task = self
            .task_config
            .0
            .get(&name)
            .ok_or("failed to get task")?
            .clone();
        println!("executing {:?}", task);

        task.run(self)?;

        Ok(())
    }

    // 更新屏幕缓存
    pub fn update_screen(&mut self) -> Result<(), String> {
        let screen = self
            .controller
            .screencap()
            .map_err(|err| format!("{err}"))?;
        self.screen_cache = Some(screen.clone());
        Ok(())
    }

    /// 获取缓存中的屏幕内容
    /// 如果没有缓存，就通过 [`AAH::update_screen`] 更新，然后再返回
    pub fn get_screen(&mut self) -> Result<image::DynamicImage, String> {
        match &self.screen_cache {
            Some(cache) => Ok(cache.clone()),
            None => {
                self.update_screen()?;
                Ok(self.screen_cache.as_ref().unwrap().clone())
            }
        }
    }

    /// 重新加载 resources 中的配置
    pub fn reload_resources(&mut self) -> Result<(), String> {
        let task_config = TaskConfig::load(&self.res_dir)
            .map_err(|err| format!("task config not found: {err}"))?;
        let navigate_config = NavigateConfig::load(&self.res_dir)
            .map_err(|err| format!("navigate config not found: {err}"))?;
        self.task_config = task_config;
        self.navigate_config = navigate_config;
        Ok(())
    }

    /// 从 `{res_path}/resources/templates/1920x1080` 目录中根据文件名称获取模板
    /// - `name` 为完整文件名
    pub fn get_template<S: AsRef<str>>(&self, name: S) -> Result<image::DynamicImage, String> {
        let name = name.as_ref();
        let path = self.res_dir.join("templates").join("1920x1080").join(name);
        let image = image::open(path).map_err(|err| format!("template not found: {err}"))?;
        Ok(image)
    }

    /// 截取当前帧的屏幕内容，分析部署卡片，返回 [`DeployAnalyzerOutput`]
    pub fn analyze_deploy(&self) -> Result<DeployAnalyzerOutput, String> {
        let mut analyzer = DeployAnalyzer;
        analyzer.analyze(self)
    }

    /// 获取所有任务名称
    pub fn get_tasks(&self) -> Vec<String> {
        self.task_config.0.keys().map(|s| s.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_get_tasks() {
        let aah = AAH::connect("127.0.0.1:16384", "../../resources").unwrap();
        println!("{:?}", aah.get_tasks());
    }

    fn save_screenshot<P: AsRef<Path>, S: AsRef<str>>(path: P, name: S) {
        let path = path.as_ref();
        let name = name.as_ref();

        let target_path = path.join(name);
        println!("saving screenshot to {:?}", target_path);

        let mut aah = AAH::connect("127.0.0.1:16384", "../../resources").unwrap();

        aah.update_screen().unwrap();
        let screen = aah.get_screen().unwrap();
        screen
            .save_with_format(target_path, image::ImageFormat::Png)
            .unwrap();
    }

    #[test]
    fn foo() {
        // let aah = Mutex::new(AAH::connect("127.0.0.1:16384", "../../resources").unwrap());
        let dir = "../../resources/templates/MUMU-1920x1080";
        // save_screenshot(dir, "start.png");
        // save_screenshot(dir, "wakeup.png");
        // save_screenshot(dir, "notice.png");
        // save_screenshot(dir, "main.png");
        // save_screenshot(dir, "confirm.png");
        // save_screenshot(dir, "operation-start.png");
        save_screenshot(dir, "1-4.png");
        // let dir = "../aah-resource/assets";
        // save_screenshot(dir, "LS-6_1.png");
    }
}
