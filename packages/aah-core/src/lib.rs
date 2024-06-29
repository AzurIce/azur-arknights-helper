#![feature(associated_type_defaults)]
#![feature(path_file_prefix)]

use std::{error::Error, path::Path};

use config::{navigate::NavigateConfig, task::TaskConfig};
use controller::{minitouch, Controller};
use ocrs::OcrEngine;

use crate::task::Task;

pub mod adb;
pub mod config;
pub mod controller;
pub mod task;
pub mod vision;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

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
        let dir = "../../resources/templates/MUMU-1920x1080";
        // save_screenshot(dir, "start.png");
        // save_screenshot(dir, "wakeup.png");
        // save_screenshot(dir, "notice.png");
        // save_screenshot(dir, "main.png");
        save_screenshot(dir, "mission.png");
    }
}

/// AAH 的实例
pub struct AAH {
    /// [`controller`] 承担设备控制相关操作（比如触摸、截图等）
    pub controller: Box<dyn Controller>,
    /// 由 `tasks.toml` 和 `tasks` 目录加载的任务配置
    pub task_config: TaskConfig,
    /// 由 `navigates.toml` 加载的导航配置
    pub navigate_config: NavigateConfig,
    pub ocr_engine: Option<OcrEngine>,
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
        let task_config =
            TaskConfig::load(&res_dir).map_err(|err| format!("task config not found: {err}"))?;
        let navigate_config = NavigateConfig::load(&res_dir)
            .map_err(|err| format!("navigate config not found: {err}"))?;
        // let controller = Box::new(AdbInputController::connect(serial)?);
        let controller = Box::new(minitouch::MiniTouchController::connect(serial)?);
        Ok(Self {
            controller,
            task_config,
            navigate_config,
            ocr_engine: None, // Some(try_init_ocr_engine()?)
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
}
