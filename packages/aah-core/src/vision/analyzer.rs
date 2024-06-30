use crate::{adb::Device, controller::Controller, AAH};

pub mod depot;
// pub mod squad;
pub mod deploy;
pub mod template_match;
pub mod multi_template_match;

pub trait Analyzer {
    type Output;
    fn analyze(&mut self, aah: &AAH) -> Result<Self::Output, String>;
}
