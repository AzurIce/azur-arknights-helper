use aah_controller::Controller;
use serde::{Deserialize, Serialize};

use crate::{Core, TaskRecipe};

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

impl<T, C> TaskRecipe<T> for Press
where
    C: Controller,
    T: Core<Controller = C>,
{
    type Res = ();
    fn run(&self, aah: &T) -> anyhow::Result<Self::Res> {
        match self.key {
            Key::Esc => aah.controller().press_esc(),
            Key::Home => aah.controller().press_home(),
        }
        .map_err(|err| anyhow::anyhow!("controller error: {:?}", err))
    }
}
