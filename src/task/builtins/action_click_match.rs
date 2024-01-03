use serde::{Deserialize, Serialize};

use crate::{
    controller::Controller,
    task::{
        wrapper::{GenericTaskWrapper, TaskWrapper},
        ExecResult, Task, match_task::MatchTask,
    },
};

#[cfg(test)]
mod test {
    use crate::task::{wrapper::GenericTaskWrapper, match_task::MatchTask};

    use super::*;

    #[test]
    fn test_serde() {
        // Without wrapper
        {
            let task = ActionClickMatch::new(MatchTask::Template("EnterMissionMistCity.png".to_string()), None);
            let task = toml::to_string_pretty(&task).unwrap();
            println!("{:?}", task);
            let task = toml::from_str::<ActionClickMatch>(&task).unwrap();
            println!("{:?}", task);
        }
        // With wrapper
        {
            let task = ActionClickMatch::new(MatchTask::Template("EnterMissionMistCity.png".to_string()), Some(GenericTaskWrapper::default()));
            let task = toml::to_string_pretty(&task).unwrap();
            println!("{:?}", task);
            let task = toml::from_str::<ActionClickMatch>(&task).unwrap();
            println!("{:?}", task);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionClickMatch {
    match_task: MatchTask,
    wrapper: Option<GenericTaskWrapper>,
}

impl ActionClickMatch {
    pub fn new(match_task: MatchTask, wrapper: Option<GenericTaskWrapper>) -> Self {
        Self {
            match_task,
            wrapper,
        }
    }
}

impl Task for ActionClickMatch {
    type Err = String;
    fn run(&self, controller: &Controller) -> Result<Self::Res, Self::Err> {
        let task = || {
            controller
                .click_in_rect(self.match_task.result(controller)?)
                .map_err(|err| format!("controller error: {:?}", err))
        };

        if let Some(wrapper) = &self.wrapper {
            wrapper.run(task)
        } else {
            task()
        }
    }
}
