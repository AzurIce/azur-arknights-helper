use image::DynamicImage;

use crate::{vision::analyzer::battle::BattleAnalyzerOutput, AAH};

pub mod builtins;
pub mod copilot;
pub mod match_task;
pub mod wrapper;

pub trait Task {
    type Res = ();
    type Err = ();
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err>;
}

/// 任务事件
///
/// - `Log(String)`: log 信息
/// - `Img(DynamicImage)`: 标记过的图片
pub enum TaskEvt {
    Log(String),
    AnnotatedImg(DynamicImage),
    BattleAnalyzerRes(BattleAnalyzerOutput),
}
