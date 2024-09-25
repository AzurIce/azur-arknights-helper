use std::{thread, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{task::Runnable, AAH};

use super::Action;

// pub trait Navigatable {
//     fn navigate_in(&self, aah: &crate::AAH);
//     fn navigate_out(&self, aah: &crate::AAH);
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NavigateIn {
    to: String,
}

impl NavigateIn {
    pub fn new(to: impl AsRef<str>) -> Self {
        Self {
            to: to.as_ref().to_string(),
        }
    }
}

impl Into<Action> for NavigateIn {
    fn into(self) -> Action {
        Action::NavigateIn(self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NavigateOut {
    from: String,
}

impl NavigateOut {
    pub fn new(from: impl AsRef<str>) -> Self {
        Self {
            from: from.as_ref().to_string(),
        }
    }
}

impl Into<Action> for NavigateOut {
    fn into(self) -> Action {
        Action::NavigateOut(self)
    }
}

impl Runnable for NavigateIn {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        thread::sleep(Duration::from_secs_f32(0.5)); // TODO: get this elegant (refactor the structure)
        let navigate = aah
            .resource
            .get_navigate(&self.to)
            .ok_or(format!("navigate {} not found", self.to))?;

        navigate.enter.run(aah).map(|_| ())
    }
}

impl Runnable for NavigateOut {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        thread::sleep(Duration::from_secs_f32(0.5)); // TODO: get this elegant (refactor the structure)
        let navigate = aah
            .resource
            .get_navigate(&self.from)
            .ok_or(format!("navigate {} not found", self.from))?;

        navigate.exit.run(aah).map(|_| ())
    }
}
