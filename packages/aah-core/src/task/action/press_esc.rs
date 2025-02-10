use serde::{Deserialize, Serialize};

use crate::{task::Runnable, AAH};

use super::Action;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressEsc;

impl Into<Action> for PressEsc {
    fn into(self) -> Action {
        Action::PressEsc(self)
    }
}

impl Runnable for PressEsc {
    type Res = ();
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        aah.controller
            .press_esc()
            .map_err(|err| format!("controller error: {:?}", err))
    }
}
