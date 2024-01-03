use std::{
    any::Any,
    fmt::{format, Debug},
    path::Path,
    thread::sleep,
    time::{self, Duration},
};

use image::{math::Rect, GenericImage};
use imageproc::point::Point;
use serde::{Deserialize, Serialize};

use crate::{
    config::{
        navigate::NavigateConfig,
        task::{self, BuiltinTask, TaskConfig},
    },
    controller::{self, Controller},
    vision::matcher::Matcher,
};

pub mod wrapper;
pub mod builtins;
pub mod match_task;

use wrapper::{GenericTaskWrapper, TaskWrapper};

#[cfg(test)]
mod test {
    use std::error::Error;

    use super::*;
    use crate::controller::Controller;

    // #[test]
    // fn test_click_match_task() -> Result<(), Box<dyn Error>> {
    //     let controller = Controller::connect("127.0.0.1:16384")?;
    //     let click_match_task =
    //         ActionTask::ClickMatch(MatchTask::Template("EnterInfrastMistCity.png".to_string()));
    //     // cost: 1.0240299, min: 1.6783588, max: 7450.5957, loc: (1869, 1146)
    //     click_match_task.run(&controller)?;
    //     Ok(())
    // }

    // #[test]
    // fn test_deserialize_with_wrapper() {
    //     // With wrapper field, and use integer for float
    //     let task = r#"
    //         arg1 = "string"
            
    //         [wrapper]
    //             delay = 6
    //             retry = 2
    //             repeat = 1
    //     "#;
    //     let task = toml::from_str::<MyTask<GenericTaskWrapper>>(&task);
    //     println!("{:?}", task)
    // }

    // #[test]
    // fn test_deserialize_without_wrapper() {
    //     // Without wrapper field
    //     let task = r#"
    //         arg1 = "string"
    //     "#;
    //     let task = toml::from_str::<MyTask<GenericTaskWrapper>>(&task);
    //     println!("{:?}", task)
    // }

    // #[test]
    // fn test_serde() {
    //     let task = MyTask {
    //         arg1: "string".to_string(),
    //         wrapper: GenericTaskWrapper::default(),
    //     };

    //     let str = toml::to_string_pretty(&task).unwrap();
    //     println!("{str:?}");

    //     let task: MyTask<GenericTaskWrapper> = toml::from_str(&str).unwrap();
    //     println!("{task:?}");
    // }

}

pub trait Task {
    type Res = ();
    type Err = ();
    fn run(&self, controller: &Controller) -> Result<Self::Res, Self::Err>;
}

/// 任务 Trait
pub trait Exec: std::fmt::Debug {
    fn run(&self, controller: &Controller) -> Result<(), String>;
}

/// 带返回结果的任务 Trait
pub trait ExecResult: std::fmt::Debug {
    type Type = ();
    fn result(&self, controller: &Controller) -> Result<Self::Type, String>;
}

impl<T: ExecResult> Exec for T {
    fn run(&self, controller: &Controller) -> Result<(), String> {
        self.result(controller).map(|_| ())
    }
}
