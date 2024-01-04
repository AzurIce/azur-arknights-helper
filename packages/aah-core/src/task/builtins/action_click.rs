use serde::{Deserialize, Serialize};

use crate::{
    task::{
        wrapper::{GenericTaskWrapper, TaskWrapper},
        Task,
    }, AAH,
};

#[cfg(test)]
mod test {
    use crate::task::wrapper::GenericTaskWrapper;

    use super::*;

    #[test]
    fn test_serde() {
        // Without wrapper
        {
            let task = ActionClick::new(0, 0, None);
            let task = toml::to_string_pretty(&task).unwrap();
            println!("{:?}", task);
            let task = toml::from_str::<ActionClick>(&task).unwrap();
            println!("{:?}", task);
        }
        // With wrapper
        {
            let task = ActionClick::new(0, 0, Some(GenericTaskWrapper::default()));
            let task = toml::to_string_pretty(&task).unwrap();
            println!("{:?}", task);
            let task = toml::from_str::<ActionClick>(&task).unwrap();
            println!("{:?}", task);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionClick {
    x: u32,
    y: u32,
    wrapper: Option<GenericTaskWrapper>,
}

impl ActionClick {
    pub fn new(x: u32, y: u32, wrapper: Option<GenericTaskWrapper>) -> Self {
        Self { x, y, wrapper }
    }
}

impl Task for ActionClick {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        let task = || {
            aah.controller
                .click(self.x, self.y)
                .map_err(|err| format!("controller error: {:?}", err))
        };

        if let Some(wrapper) = &self.wrapper {
            wrapper.run(task)
        } else {
            task()
        }
    }
}
