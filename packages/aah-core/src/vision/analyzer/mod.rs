use crate::AAH;

// pub mod depot;
// pub mod formation;
pub mod battle;
pub mod multi_match;
pub mod single_match;
mod ocr;

/// [`Analyzer`] 接收图像，返回分析结果 [`Analyzer::Output`]
pub trait Analyzer {
    type Output;
    fn analyze(&mut self, aah: &AAH) -> Result<Self::Output, String>;
}
