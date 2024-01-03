use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    config::{
        navigate::NavigateConfig,
        task::{BuiltinTask, TaskConfig},
    },
    controller::Controller,
    task::{
        wrapper::{GenericTaskWrapper, TaskWrapper},
        ExecResult, Task,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Navigate {
    NavigateIn(String),
    NavigateOut(String),
}

impl Task for Navigate {
    type Err = String;
    fn run(&self, controller: &crate::controller::Controller) -> Result<Self::Res, Self::Err> {
        let name = match self {
            Navigate::NavigateIn(name) => name,
            Navigate::NavigateOut(name) => name,
        };

        let navigate_config = NavigateConfig::load().map_err(|err| format!("{:?}", err))?;
        let navigate = navigate_config.get_navigate(name)?;

        let task = match self {
            Navigate::NavigateIn(task) => navigate.enter_task,
            Navigate::NavigateOut(name) => navigate.exit_task,
        };
        task.run(controller).map(|_| ())
    }
}
