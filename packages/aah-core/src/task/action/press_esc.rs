use crate::{task::Runnable, AAH};

pub struct PressEsc;

impl Runnable for PressEsc {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        aah.controller
            .press_esc()
            .map_err(|err| format!("controller error: {:?}", err))
    }
}
