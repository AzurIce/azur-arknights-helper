use crate::{task::Runnable, AAH};

pub struct Click {
    x: u32,
    y: u32,
}

impl Click {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl Runnable for Click {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        aah.controller
            .click(self.x, self.y)
            .map_err(|err| format!("controller error: {:?}", err))
    }
}