use serde::{Deserialize, Serialize};

use crate::{
    task::{wrapper::GenericTaskWrapper, Task},
    AAH,
};

use super::BuiltinTask;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Multi {
    tasks: Vec<BuiltinTask>,
    #[serde(default = "default_fail_fast")]
    fail_fast: bool,
    wrapper: Option<GenericTaskWrapper>,
}

fn default_fail_fast() -> bool {
    true
}

impl Multi {
    pub fn new(tasks: Vec<BuiltinTask>, fail_fast: bool, wrapper: Option<GenericTaskWrapper>) -> Self {
        Self {
            tasks,
            fail_fast,
            wrapper,
        }
    }
}

impl Task for Multi {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        let mut res = Ok(());
        for task in &self.tasks {
            res = task.run(aah).map(|_| ());
            println!("{:?}", res);
            if res.is_err() && self.fail_fast {
                break;
            }
        }
        res.map_err(|err| format!("[Multi]: error when executing task {:?}: {:?}", self, err))
    }
}
