use serde::{Deserialize, Serialize};

use crate::{
    task::{
        wrapper::{GenericTaskWrapper, TaskWrapper},
        Task,
    },
    AAH,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionPressEsc {
    wrapper: Option<GenericTaskWrapper>,
}

impl ActionPressEsc {
    pub fn new(wrapper: Option<GenericTaskWrapper>) -> Self {
        Self { wrapper }
    }
}

impl Task for ActionPressEsc {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        let task = || {
            aah.controller
                .press_esc()
                .map_err(|err| format!("controller error: {:?}", err))
        };

        if let Some(wrapper) = &self.wrapper {
            wrapper.run(task)
        } else {
            task()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::task::wrapper::GenericTaskWrapper;

    use super::*;

    #[test]
    fn test_serde() {
        // Without wrapper
        {
            let task = ActionPressEsc::new(None);
            let task = toml::to_string_pretty(&task).unwrap();
            println!("{:?}", task);
            let task = toml::from_str::<ActionPressEsc>(&task).unwrap();
            println!("{:?}", task);
        }
        // With wrapper
        {
            let task = ActionPressEsc::new(Some(GenericTaskWrapper::default()));
            let task = toml::to_string_pretty(&task).unwrap();
            println!("{:?}", task);
            let task = toml::from_str::<ActionPressEsc>(&task).unwrap();
            println!("{:?}", task);
        }
    }
}
