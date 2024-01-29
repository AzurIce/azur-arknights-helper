use crate::{adb::Device, controller::Controller};

pub mod depot_analyzer;

pub trait Analyzer {
    type Output;
    fn analyze(&mut self, device: &impl Controller) -> Result<Self::Output, String>;
}