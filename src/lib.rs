use std::sync::{Arc, Mutex};

use aah_controller::Controller;


#[cfg(feature = "arknights")]
pub mod arknights;
#[cfg(feature = "android")]
pub mod android;
#[cfg(feature = "desktop")]
pub mod desktop;
pub mod resource;
pub mod task;
pub mod vision;
pub mod utils;

pub trait AahTrait {
    type Controller;
    type Resource;
    fn controller(&self) -> &Self::Controller;
    fn resource(&self) -> &Self::Resource;
}

pub trait CachedScreenCapper {
    fn screen_cache_or_cap(&self) -> anyhow::Result<image::DynamicImage>;
    fn screen_cap_and_cache(&self) -> anyhow::Result<image::DynamicImage>;
}

// pub struct GeneralAah<C: Controller, R> {
//     pub controller: C,
//     pub resource: Arc<R>,
//     screen_cache: Mutex<Option<image::DynamicImage>>,
// }

// impl<C: Controller, R> GeneralAah<C, R> {

// }


#[cfg(test)]
mod test {
    use std::{
        path::Path,
        sync::{Mutex, OnceLock},
    };

    // use resource::LocalResource;

    use super::*;
    use std::sync::Arc;

    // use crate::arknights::AAH;

    // /// An AAH instance using [`LocalResource`], and connected to `127.0.0.1:16384`
    // pub fn aah_for_test() -> AAH {
    //     let resource = LocalResource::load("../../resources").unwrap();
    //     AAH::connect("127.0.0.1:16384", Arc::new(resource.into())).unwrap()
    // }

    // #[test]
    // fn foo() {
    //     let resource = Arc::new(LocalResource::load("../../resources").unwrap().into());
    //     let mut aah = AAH::connect("127.0.0.1:16384", resource).unwrap();
    //     aah.register_task_evt_handler(|evt| {
    //         if let TaskEvt::BattleAnalyzerRes(res) = evt {
    //             println!("{:?}", res);
    //         }
    //     });
    //     aah.start_battle_analyzer()
    // }

    // #[test]
    // fn test_get_tasks() {
    //     static S: OnceLock<Mutex<Option<AAH>>> = OnceLock::new();
    //     let _ = &S;
    //     let resource = Arc::new(LocalResource::load("../../resources").unwrap().into());
    //     let aah = AAH::connect("127.0.0.1:16384", resource).unwrap();
    //     println!("{:?}", aah.resource.get_tasks());
    // }

    // fn save_screenshot<P: AsRef<Path>, S: AsRef<str>>(aah: &mut AAH, path: P, name: S) {
    //     let path = path.as_ref();
    //     let name = name.as_ref();

    //     let target_path = path.join(name);
    //     println!("saving screenshot to {:?}", target_path);

    //     // aah.update_screen().unwrap();
    //     let screen = aah.get_screen().unwrap();
    //     screen
    //         .save_with_format(target_path, image::ImageFormat::Png)
    //         .unwrap();
    // }

    // #[test]
    // fn screenshot() {
    //     let resource = Arc::new(LocalResource::load("../../resources").unwrap().into());
    //     let mut aah = AAH::connect("127.0.0.1:16384", resource).unwrap();
    //     let dir = "../../resources/templates/MUMU-1920x1080";
    //     // save_screenshot(dir, "start.png");
    //     // save_screenshot(dir, "wakeup.png");
    //     // save_screenshot(dir, "notice.png");
    //     // save_screenshot(dir, "main.png");
    //     // save_screenshot(dir, "confirm.png");
    //     save_screenshot(&mut aah, dir, "1-4_deploying_direction.png");
    //     // let dir = "/Volumes/Data/Dev/AahAI/dataset/1-4/img";
    //     // for i in 0..10 {
    //     //     save_screenshot(&mut aah, dir, format!("{i}.png"));
    //     //     sleep(Duration::from_secs_f32(0.2))
    //     // }
    //     // let dir = "../aah-resource/assets";
    //     // save_screenshot(dir, "LS-6_1.png");
    // }
}
