use std::{thread, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{task::Runnable, AAH};

pub trait Navigatable {
    fn navigate_in(&self, aah: &crate::AAH);
    fn navigate_out(&self, aah: &crate::AAH);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Navigate {
    NavigateIn(String),
    NavigateOut(String),
}

impl Runnable for Navigate {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        thread::sleep(Duration::from_secs_f32(0.5)); // TODO: get this elegant (refactor the structure)
        let name = match self {
            Navigate::NavigateIn(name) => name,
            Navigate::NavigateOut(name) => name,
        };

        let navigate = aah
            .resource
            .get_navigate(name)
            .ok_or(format!("navigate {} not found", name))?;

        let action = match self {
            Navigate::NavigateIn(_) => &navigate.enter,
            Navigate::NavigateOut(_) => &navigate.exit,
        };
        action.run(aah).map(|_| ())
    }
}
