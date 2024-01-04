#![feature(associated_type_defaults)]
#![feature(path_file_prefix)]

use std::error::Error;

use config::{navigate::NavigateConfig, task::TaskConfig};
use controller::Controller;

use crate::task::Task;

pub mod adb;
pub mod config;
pub mod controller;
pub mod task;
pub mod vision;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

pub struct AAH {
    pub controller: Controller,
    pub task_config: TaskConfig,
    pub navigate_config: NavigateConfig,
}

impl AAH {
    pub fn connect<S: AsRef<str>>(serial: S) -> Result<Self, Box<dyn Error>> {
        let task_config = TaskConfig::load("./resources")?;
        let navigate_config = NavigateConfig::load("./resources")?;
        let controller = Controller::connect(serial)?;
        Ok(Self {
            controller,
            task_config,
            navigate_config,
        })
    }

    pub fn run_task<S: AsRef<str>>(&self, name: S) -> Result<(), String> {
        let name = name.as_ref().to_string();

        let task = self.task_config
            .0
            .get(&name)
            .ok_or("failed to get task")?
            .clone();
        println!("executing {:?}", task);

        task.run(self)?;

        Ok(())
    }
}
