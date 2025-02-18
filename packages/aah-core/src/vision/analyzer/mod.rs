//! Analyzer 所做的事情为通过对应的 Aah 进行操作、截图、计算等操作最终返回一个结果
pub mod matching;
pub mod multi_match;
pub mod single_match;

pub trait Analyzer<T> {
    type Res;
    fn analyze(&mut self, core: &T) -> anyhow::Result<Self::Res>;
}
