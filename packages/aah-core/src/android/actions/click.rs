use aah_controller::Controller;
use serde::{Deserialize, Serialize};

use crate::{Core, TaskRecipe};

use super::ActionSet;

/// An action for clicking the specific coordinate on the screen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Click {
    x: u32,
    y: u32,
}

impl Into<ActionSet> for Click {
    fn into(self) -> ActionSet {
        ActionSet::Click(self)
    }
}

impl Click {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl<T, C> TaskRecipe<T> for Click
where
    C: Controller,
    T: Core<Controller = C>,
{
    type Res = ();
    fn run(&self, runner: &T) -> anyhow::Result<Self::Res> {
        runner
            .controller()
            .click(self.x, self.y)
            .map_err(|err| anyhow::anyhow!("controller error: {:?}", err))
    }
}
