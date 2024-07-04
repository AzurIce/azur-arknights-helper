use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    task::{
        wrapper::{GenericTaskWrapper, TaskWrapper},
        Task,
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
            let task = ActionSwipe::new((0, 0), (10, 10), 1.0, None);
            let task = toml::to_string_pretty(&task).unwrap();
            println!("{:?}", task);
            let task = toml::from_str::<ActionSwipe>(&task).unwrap();
            println!("{:?}", task);
        }
        // With wrapper
        {
            let task = ActionSwipe::new((0, 0), (10, 10), 1.0, Some(GenericTaskWrapper::default()));
            let task = toml::to_string_pretty(&task).unwrap();
            println!("{:?}", task);
            let task = toml::from_str::<ActionSwipe>(&task).unwrap();
            println!("{:?}", task);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionSwipe {
    p1: (u32, u32),
    p2: (i32, i32),
    #[serde(default = "default_duration")]
    duration: f32,
    wrapper: Option<GenericTaskWrapper>,
}

fn default_duration() -> f32 {
    1.0
}

impl ActionSwipe {
    pub fn new(
        p1: (u32, u32),
        p2: (i32, i32),
        duration: f32,
        wrapper: Option<GenericTaskWrapper>,
    ) -> Self {
        Self {
            p1,
            p2,
            duration,
            wrapper,
        }
    }
}

impl Task for ActionSwipe {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        let task = || {
            aah.controller
                .swipe_scaled(self.p1, self.p2, Duration::from_secs_f32(self.duration))
                .map_err(|err| format!("controller error: {:?}", err))
        };

        if let Some(wrapper) = &self.wrapper {
            wrapper.run(task)
        } else {
            task()
        }
    }
}
