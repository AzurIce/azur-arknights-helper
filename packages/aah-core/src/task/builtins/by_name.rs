use serde::{Deserialize, Serialize};

use crate::task::{
    wrapper::{GenericTaskWrapper, TaskWrapper},
    Task,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ByName {
    name: String,
    wrapper: Option<GenericTaskWrapper>,
}

impl ByName {
    pub fn new<S: AsRef<str>>(name: S, wrapper: Option<GenericTaskWrapper>) -> Self {
        let name = name.as_ref().to_string();
        ByName { name, wrapper }
    }
}

impl Task for ByName {
    type Err = String;
    fn run(&self, aah: &crate::AAH) -> Result<Self::Res, Self::Err> {
        let exec = || aah.run_task(&self.name);
        if let Some(wrapper) = &self.wrapper {
            wrapper.run(exec)
        } else {
            exec()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::AAH;

    #[test]
    fn test_name_task() {
        let aah = AAH::connect("127.0.0.1:16384", "../resources").unwrap();
        aah.run_task("wakeup").unwrap();
    }
}
