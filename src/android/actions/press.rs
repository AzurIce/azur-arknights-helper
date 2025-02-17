use aah_controller::Controller;
use serde::{Deserialize, Serialize};

use crate::task::Runnable;

use super::ActionSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Key {
    Esc,
    Home,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Press {
    key: Key,
}

impl Press {
    pub fn esc() -> Self {
        Self { key: Key::Esc }
    }
    pub fn home() -> Self {
        Self { key: Key::Home }
    }
}

impl Into<ActionSet> for Press {
    fn into(self) -> ActionSet {
        ActionSet::Press(self)
    }
}

impl<T: Controller> Runnable<T> for Press {
    type Res = ();
    fn run(&self, runner: &T) -> anyhow::Result<Self::Res> {
        match self.key {
            Key::Esc => runner.press_esc(),
            Key::Home => runner.press_home(),
        }
        .map_err(|err| anyhow::anyhow!("controller error: {:?}", err))
    }
}
