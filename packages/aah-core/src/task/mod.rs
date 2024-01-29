use crate::AAH;

pub mod builtins;
pub mod match_task;
pub mod wrapper;

pub trait Task {
    type Res = ();
    type Err = ();
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err>;
}
