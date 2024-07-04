use serde::{Deserialize, Serialize};

use crate::{
    task::{
        wrapper::{GenericTaskWrapper, TaskWrapper},
        Task, TaskEvt,
    },
    AAH,
};

#[cfg(test)]
mod test {
    use crate::task::wrapper::GenericTaskWrapper;

    use super::*;

    #[test]
    fn test_serde() {
        // Without wrapper
        {
            let task = ActionPressHome::new(None);
            let task = toml::to_string_pretty(&task).unwrap();
            println!("{:?}", task);
            let task = toml::from_str::<ActionPressHome>(&task).unwrap();
            println!("{:?}", task);
        }
        // With wrapper
        {
            let task = ActionPressHome::new(Some(GenericTaskWrapper::default()));
            let task = toml::to_string_pretty(&task).unwrap();
            println!("{:?}", task);
            let task = toml::from_str::<ActionPressHome>(&task).unwrap();
            println!("{:?}", task);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionPressHome {
    wrapper: Option<GenericTaskWrapper>,
}

impl ActionPressHome {
    pub fn new(wrapper: Option<GenericTaskWrapper>) -> Self {
        Self { wrapper }
    }
}

impl Task for ActionPressHome {
    type Err = String;
    fn run(&self, aah: &AAH, on_task_evt: impl Fn(TaskEvt)) -> Result<Self::Res, Self::Err> {
        let task = || {
            aah.controller
                .press_home()
                .map_err(|err| format!("controller error: {:?}", err))
        };

        if let Some(wrapper) = &self.wrapper {
            wrapper.run(task)
        } else {
            task()
        }
    }
}
