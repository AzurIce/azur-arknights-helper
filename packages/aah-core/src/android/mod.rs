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
    Core, TaskRecipe,
};

pub mod actions;

pub use actions::ActionSet;

/// 通用 Android AAH
///
/// ActionSet: 见 [`actions::AndroidActionSet`]
pub struct GeneralAndroidCore {
    controller: Box<dyn Controller>,
    resource: Arc<GeneralAahResource<actions::ActionSet>>,
    screen_cache: Mutex<Option<image::DynamicImage>>,
}

impl Core for GeneralAndroidCore {
    type Controller = Box<dyn Controller>;
    type Resource = GeneralAahResource<actions::ActionSet>;

    fn resource(&self) -> &Self::Resource {
        &self.resource
    }

    fn controller(&self) -> &Self::Controller {
        &self.controller
    }
}

impl GeneralAndroidCore {
    /// 连接到 `serial` 指定的设备（`serial` 就是 `adb devices` 里的序列号）
    ///
    /// - `serial`: 设备的序列号
    /// - `res_dir`: 资源目录的路径
    pub fn connect(
        serial: impl AsRef<str>,
        resource: GeneralAahResource<actions::ActionSet>,
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
        resource: GeneralAahResource<actions::ActionSet>,
    ) -> Result<Self, anyhow::Error> {
        let controller = Box::new(AdbController::connect(serial)?);

        Self::new(controller, resource)
    }

    fn new(
        controller: Box<dyn Controller + Sync + Send>,
        resource: GeneralAahResource<actions::ActionSet>,
    ) -> Result<Self, anyhow::Error> {
        let resource = Arc::new(resource);
        Ok(Self {
            controller,
            resource,
            screen_cache: Mutex::new(None),
        })
    }

    pub fn run_task(&self, name: impl AsRef<str>) -> anyhow::Result<()> {
        let name = name.as_ref().to_string();
        info!("running task: {}...", name);
        let task = self
            .resource
            .get_task(name)
            .ok_or(anyhow::anyhow!("failed to get task"))?;

        task.run(self)
    }

    /// Get screen cache or capture one. This is for internal analyzer use
    pub fn screen_cache_or_cap(&self) -> anyhow::Result<image::DynamicImage> {
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

    pub fn screen_cap_and_cache(&self) -> anyhow::Result<image::DynamicImage> {
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

impl ResRoot for GeneralAndroidCore {
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
        let aah = GeneralAndroidCore::connect("127.0.0.1:16384", resource).unwrap();
        aah.run_task("arknights_wakeup").unwrap();
    }
}
