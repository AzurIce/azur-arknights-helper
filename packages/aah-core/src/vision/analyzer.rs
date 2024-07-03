use serde::Serialize;

use crate::AAH;

pub mod depot;
// pub mod squad;
pub mod deploy;
pub mod single_match;
pub mod multi_match;

/// [`Analyzer`] 接收图像，返回分析结果 [`Analyzer::Output`]
pub trait Analyzer {
    type Output;
    fn analyze(&mut self, aah: &AAH) -> Result<Self::Output, String>;
}
