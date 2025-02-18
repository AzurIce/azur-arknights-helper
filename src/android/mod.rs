use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use aah_controller::{
    android::{AahController, AdbController},
    Controller,
};
use anyhow::Result;
use log::info;

use crate::{
    resource::{GeneralAahResource, GetTask, ResRoot},
    task::{Runnable, Runner},
    CachedScreenCapper,
};

pub mod actions;

pub use actions::ActionSet;

/// 通用 Android AAH
///
/// ActionSet: 见 [`actions::AndroidActionSet`]
pub struct GeneralAndroidAah {
    pub controller: Box<dyn Controller + Sync + Send>,
    pub resource: Arc<GeneralAahResource<actions::ActionSet>>,
    screen_cache: Mutex<Option<image::DynamicImage>>,
}

impl Controller for GeneralAndroidAah {
    fn click(&self, x: u32, y: u32) -> Result<()> {
        self.controller.click(x, y)
    }
    fn swipe(
        &self,
        start: (u32, u32),
        end: (i32, i32),
        duration: std::time::Duration,
        slope_in: f32,
        slope_out: f32,
    ) -> Result<()> {
        self.controller
            .swipe(start, end, duration, slope_in, slope_out)
    }
    fn screen_size(&self) -> (u32, u32) {
        self.controller.screen_size()
    }
    fn screencap(&self) -> Result<image::DynamicImage> {
        self.controller.screencap()
    }
    fn raw_screencap(&self) -> Result<Vec<u8>> {
        self.controller.raw_screencap()
    }
    fn press_esc(&self) -> Result<()> {
        self.controller.press_esc()
    }
    fn press_home(&self) -> Result<()> {
        self.controller.press_home()
    }
}

impl GeneralAndroidAah {
    /// 连接到 `serial` 指定的设备（`serial` 就是 `adb devices` 里的序列号）
    ///
    /// - `serial`: 设备的序列号
    /// - `res_dir`: 资源目录的路径
    pub fn connect(
        serial: impl AsRef<str>,
        resource: Arc<GeneralAahResource<actions::ActionSet>>,
    ) -> Result<Self, anyhow::Error> {
        let controller = Box::new(AahController::connect(serial)?);

        Self::new(controller, resource)
    }

    /// 连接到 `serial` 指定的设备（`serial` 就是 `adb devices` 里的序列号）
    /// 使用 ADB 控制器
    ///
    /// - `serial`: 设备的序列号
    /// - `res_dir`: 资源目录的路径
    pub fn connect_with_adb_controller(
        serial: impl AsRef<str>,
        resource: Arc<GeneralAahResource<actions::ActionSet>>,
    ) -> Result<Self, anyhow::Error> {
        let controller = Box::new(AdbController::connect(serial)?);

        Self::new(controller, resource)
    }

    fn new(
        controller: Box<dyn Controller + Sync + Send>,
        resource: Arc<GeneralAahResource<actions::ActionSet>>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            resource,
            controller,
            screen_cache: Mutex::new(None),
        })
    }
}

impl Runner for GeneralAndroidAah {
    fn run_task(&self, name: impl AsRef<str>) -> anyhow::Result<()> {
        let name = name.as_ref().to_string();
        info!("running task: {}...", name);
        let task = self
            .resource
            .get_task(name)
            .ok_or(anyhow::anyhow!("failed to get task"))?;

        task.run(self)
    }
}

impl CachedScreenCapper for GeneralAndroidAah {
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

impl ResRoot for GeneralAndroidAah {
    fn res_root(&self) -> &Path {
        self.resource.root.as_path()
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::resource::Load;

    use super::*;

    #[test]
    fn foo() {
        let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let root = Path::new(&root);

        let resource = GeneralAahResource::load(root.join("test/android_resources")).unwrap();
        let resource = Arc::new(resource.into());
        let aah = GeneralAndroidAah::connect("127.0.0.1:16384", resource).unwrap();
        aah.run_task("arknights_wakeup").unwrap();
    }
}
