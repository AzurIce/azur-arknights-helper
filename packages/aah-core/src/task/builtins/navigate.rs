use std::{thread, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{task::{Task, TaskEvt}, AAH};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Navigate {
    NavigateIn(String),
    NavigateOut(String),
}

impl Task for Navigate {
    type Err = String;
    fn run(&self, aah: &AAH, on_task_evt: impl Fn(TaskEvt)) -> Result<Self::Res, Self::Err> {
        thread::sleep(Duration::from_secs_f32(0.5)); // TODO: get this elegant (refactor the structure)
        let name = match self {
            Navigate::NavigateIn(name) => name,
            Navigate::NavigateOut(name) => name,
        };

        let navigate = aah.navigate_config.get_navigate(name)?;

        let task = match self {
            Navigate::NavigateIn(_) => navigate.enter_task,
            Navigate::NavigateOut(_) => navigate.exit_task,
        };
        task.run(aah, on_task_evt).map(|_| ())
    }
}
