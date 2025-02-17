use aah_controller::Controller;
use serde::{Deserialize, Serialize};

use crate::task::Runnable;

use super::AndroidActionSet;

/// An action for clicking the specific coordinate on the screen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Click {
    x: u32,
    y: u32,
}

impl Into<AndroidActionSet> for Click {
    fn into(self) -> AndroidActionSet {
        AndroidActionSet::Click(self)
    }
}

impl Click {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl<T: Controller> Runnable<T> for Click {
    type Res = ();
    fn run(&self, runner: &T) -> anyhow::Result<Self::Res> {
        runner
            .click(self.x, self.y)
            .map_err(|err| anyhow::anyhow!("controller error: {:?}", err))
    }
}
