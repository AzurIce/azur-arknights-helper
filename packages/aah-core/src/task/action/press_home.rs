use serde::{Deserialize, Serialize};

use crate::{task::Runnable, AAH};

use super::Action;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressHome;

impl Into<Action> for PressHome {
    fn into(self) -> Action {
        Action::PressHome(self)
    }
}

impl Runnable for PressHome {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        aah.controller
            .press_home()
            .map_err(|err| format!("controller error: {:?}", err))
    }
}
