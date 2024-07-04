use image::DynamicImage;

use crate::AAH;

pub mod builtins;
pub mod match_task;
pub mod wrapper;
pub mod copilot;

pub trait Task {
    type Res = ();
    type Err = ();
    fn run(&self, aah: &AAH, on_task_evt: impl Fn(TaskEvt)) -> Result<Self::Res, Self::Err>;
}

/// 任务事件
/// 
/// - `Log(String)`: log 信息
/// - `Img(DynamicImage)`: 标记过的图片
pub enum TaskEvt {
    Log(String),
    AnnotatedImg(DynamicImage)
}