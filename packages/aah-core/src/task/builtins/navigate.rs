use serde::{Deserialize, Serialize};

use crate::{config::navigate::NavigateConfig, controller::Controller, task::Task};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Navigate {
    NavigateIn(String),
    NavigateOut(String),
}

impl Task for Navigate {
    type Err = String;
    fn run(&self, controller: &Controller) -> Result<Self::Res, Self::Err> {
        let name = match self {
            Navigate::NavigateIn(name) => name,
            Navigate::NavigateOut(name) => name,
        };

        let navigate_config = NavigateConfig::load().map_err(|err| format!("{:?}", err))?;
        let navigate = navigate_config.get_navigate(name)?;

        let task = match self {
            Navigate::NavigateIn(_) => navigate.enter_task,
            Navigate::NavigateOut(_) => navigate.exit_task,
        };
        task.run(controller).map(|_| ())
    }
}
