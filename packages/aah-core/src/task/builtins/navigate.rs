use serde::{Deserialize, Serialize};

use crate::{task::Task, AAH};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Navigate {
    NavigateIn(String),
    NavigateOut(String),
}

impl Task for Navigate {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        let name = match self {
            Navigate::NavigateIn(name) => name,
            Navigate::NavigateOut(name) => name,
        };

        let navigate = aah.navigate_config.get_navigate(name)?;

        let task = match self {
            Navigate::NavigateIn(_) => navigate.enter_task,
            Navigate::NavigateOut(_) => navigate.exit_task,
        };
        task.run(aah).map(|_| ())
    }
}
