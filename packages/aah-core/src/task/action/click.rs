use serde::{Deserialize, Serialize};

use crate::{task::Runnable, AAH};

use super::Action;

/// An action for clicking the specific coordinate on the screen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Click {
    x: u32,
    y: u32,
}

impl Into<Action> for Click {
    fn into(self) -> Action {
        Action::ActionClick(self)
    }
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
