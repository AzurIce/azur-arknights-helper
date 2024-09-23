use crate::AAH;

// pub mod depot;
// pub mod formation;
pub mod battle;
pub mod multi_match;
pub mod single_match;
pub mod levels;
// mod ocr;

/// [`Analyzer`] 可以调用 [`AAH`] 的 API，返回分析结果 [`Analyzer::Output`]
/// 
/// 与 [`Matcher`] 的核心区别就是 [`Analyzer`] 以 [`AAH`] 为输入，而 [`Matcher`] 以图片为输入。
/// 所以在同一个 [`Analyzer`] 中可以结合多个 [`Matcher`] 以及 [`Analyzer`]，且还可以进行设备操作。
pub trait Analyzer {
    type Output;
    fn analyze(&mut self, aah: &AAH) -> Result<Self::Output, String>;
}
