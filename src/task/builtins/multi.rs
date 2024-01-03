use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    config::task::{BuiltinTask, TaskConfig},
    controller::Controller,
    task::{
        wrapper::{GenericTaskWrapper, TaskWrapper},
        ExecResult, Task,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum TaskRef {
    ByInternal(BuiltinTask),
    ByName(String),
}

impl Task for TaskRef {
    type Err = String;
    fn run(&self, controller: &crate::controller::Controller) -> Result<Self::Res, Self::Err> {
        let task = match self {
            TaskRef::ByInternal(task) => task.clone(),
            TaskRef::ByName(name) => {
                let task_config = TaskConfig::load().map_err(|err|format!("{:?}", err))?;

                let task = task_config.get_task(name)?;
                task
            }
        };
        task.run(controller)
            .map(|_| ())
            .map_err(|err| format!("failed to execute"))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Multi {
    tasks: Vec<TaskRef>,
    #[serde(default = "default_fail_fast")]
    fail_fast: bool,
    wrapper: Option<GenericTaskWrapper>,
}

fn default_fail_fast() -> bool {
    true
}

impl Multi {
    pub fn new(tasks: Vec<TaskRef>, fail_fast: bool, wrapper: Option<GenericTaskWrapper>) -> Self {
        Self {
            tasks,
            fail_fast,
            wrapper,
        }
    }
}

impl Task for Multi {
    type Err = String;
    fn run(&self, controller: &Controller) -> Result<Self::Res, Self::Err> {
        let mut res = Ok(());
        for task in &self.tasks {
            res = task.run(controller).map(|_| ());
            if res.is_err() && self.fail_fast {
                break;
            }
        }
        res.map_err(|err| format!("[Multi]: error when executing task {:?}: {:?}", self, err))
    }
}
