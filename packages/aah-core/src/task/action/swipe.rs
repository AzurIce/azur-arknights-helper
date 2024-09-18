use std::time::Duration;

use crate::{task::Runnable, AAH};

pub struct Swipe {
    p1: (u32, u32),
    p2: (i32, i32),
    duration: Duration,
    slope_in: f32,
    slope_out: f32,
}

impl Swipe {
    pub fn new(
        p1: (u32, u32),
        p2: (i32, i32),
        duration: Duration,
        slope_in: f32,
        slope_out: f32,
    ) -> Self {
        Self {
            p1,
            p2,
            duration,
            slope_in,
            slope_out,
        }
    }
}

impl Runnable for Swipe {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        aah.controller
            .swipe(
                self.p1,
                self.p2,
                self.duration,
                self.slope_in,
                self.slope_out,
            )
            .map_err(|err| format!("controller error: {:?}", err))
    }
}
