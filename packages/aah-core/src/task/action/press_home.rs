use crate::{task::Runnable, AAH};

pub struct PressHome;

impl Runnable for PressHome {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        aah.controller
            .press_home()
            .map_err(|err| format!("controller error: {:?}", err))
    }
}
