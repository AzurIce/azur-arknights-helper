use std::{path::Path, time::{Duration, self}, fmt::format, thread::sleep};

use image::math::Rect;
use imageproc::point::Point;
use serde::{Deserialize, Serialize};

use crate::{
    config::{
        navigate::NavigateConfig,
        task::{self, Task, TaskConfig, TaskRef},
    },
    controller::{self, Controller},
    vision::matcher::Matcher,
};

mod auto_recruit;
mod navigate;

#[cfg(test)]
mod test {
    use std::error::Error;

    use super::*;
    use crate::controller::Controller;

    #[test]
    fn test_click_match_task() -> Result<(), Box<dyn Error>> {
        let controller = Controller::connect("127.0.0.1:16384")?;
        let click_match_task =
            ActionTask::ClickMatch(MatchTask::Template("EnterInfrastMistCity.png".to_string()));
        // cost: 1.0240299, min: 1.6783588, max: 7450.5957, loc: (1869, 1146)
        click_match_task.run(&controller)?;
        Ok(())
    }
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

impl TryInto<Box<dyn Exec>> for TaskRef {
    type Error = String;
    fn try_into(self) -> Result<Box<dyn Exec>, Self::Error> {
        match self {
            TaskRef::ByInternal(task) => {
                let task: Box<dyn Exec> = task.try_into()?;
                Ok(task)
            }
            TaskRef::ByName(name) => {
                let task_config =
                    TaskConfig::load().map_err(|err| format!("{:?}", err))?;

                let task = task_config.0.get(&name).ok_or("failed to get task by name".to_string())?;
                let task: Box<dyn Exec> = task.clone().try_into()?;
                Ok(task)
            }
        }
    }
}

impl TryInto<Box<dyn Exec>> for Task {
    type Error = String;
    fn try_into(self) -> Result<Box<dyn Exec>, String> {
        let task: Box<dyn Exec> = match self {
            Task::Multi(tasks) => {
                let mut res = vec![];

                for task in tasks {
                    res.push(task.try_into()?);
                }

                Box::new(MultiTask::new(res))
            }
            Task::ActionPressEsc => Box::new(ActionTask::PressEsc),
            Task::ActionPressHome => Box::new(ActionTask::PressHome),
            Task::ActionClick(x, y) => Box::new(ActionTask::Click(x, y)),
            Task::ActionClickMatch(match_task) => Box::new(ActionTask::ClickMatch(match_task)),
            Task::ActionSwipe(p1, p2) => Box::new(ActionTask::Swipe(p1, p2)),
            Task::NavigateIn(name) => {
                let navigate_config = NavigateConfig::load().map_err(|err| format!("{:?}", err))?;
                let navigate = navigate_config.0.get(&name).ok_or("failed to get navigate by name".to_string())?;
                let task: Box<dyn Exec> = navigate.enter_task.clone().try_into()?;
                task
            },
            Task::NavigateOut(name) => {
                let navigate_config = NavigateConfig::load().map_err(|err| format!("{:?}", err))?;
                let navigate = navigate_config.0.get(&name).ok_or("failed to get navigate by name".to_string())?;
                let task: Box<dyn Exec> = navigate.exit_task.clone().try_into()?;
                task
            },
        };
        Ok(task)
    }
}



/// 动作任务
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionTask {
    PressEsc,
    PressHome,
    Click(u32, u32),
    ClickMatch(MatchTask),
    Swipe((u32, u32), (u32, u32)),
}

impl ExecResult for ActionTask {
    type Type = ();
    fn result(&self, controller: &Controller) -> Result<(), String> {
        match self {
            ActionTask::PressEsc => controller.press_esc(),
            ActionTask::PressHome => controller.press_home(),
            ActionTask::Click(x, y) => controller.click(x.clone(), y.clone()),
            ActionTask::ClickMatch(match_task) => {
                controller.click_in_rect(match_task.result(controller)?)
            }
            ActionTask::Swipe(p1, p2) => {
                controller.swipe(p1.clone(), p2.clone(), Duration::from_secs(1))
            }
        }
        .map_err(|err| format!("{:?}", err))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "template")]
pub enum MatchTask {
    Template(String), // template_filename
    Ocr(String),      // text
}
// TODO: add optional roi field

impl ExecResult for MatchTask {
    type Type = Rect;
    fn result(&self, controller: &Controller) -> Result<Self::Type, String> {
        let image = controller.screencap().map_err(|err| format!("{:?}", err))?;
        let image = image.to_luma32f();

        let matcher = match self {
            Self::Template(template_filename) => {
                let template =
                    image::open(Path::new("./resources/templates/").join(template_filename))
                        .map_err(|err| format!("{:?}", err))?
                        .to_luma32f();
                Matcher::Template { image, template }
            }
            Self::Ocr(text) => Matcher::Ocr(text.to_string()),
        };

        let res = matcher.result().ok_or("match failed".to_string())?;
        Ok(res)
    }
}

#[derive(Debug)]
pub struct MultiTask {
    tasks: Vec<Box<dyn Exec>>,
}

impl MultiTask {
    pub fn new(tasks: Vec<Box<dyn Exec>>) -> Self {
        Self { tasks }
    }
}

impl Exec for MultiTask {
    // TODO: add faild option for generic task
    fn run(&self, controller: &Controller) -> Result<(), String> {
        for task in &self.tasks {
            println!("[Multitask]: execiting {:?}", task);
            match task.run(controller) {
                Ok(_) => (),
                Err(err) => println!("[Multitask]: error, {:?}", err)
            }
            sleep(Duration::from_millis(500))
        }
        Ok(())
    }
}

// 若任何 Match 失败则失败
// 成功返回所有匹配 Rect
#[derive(Debug)]
pub struct AndTask {
    tasks: Vec<Box<dyn Exec>>,
}

impl AndTask {
    pub fn new(tasks: Vec<Box<dyn Exec>>) -> Self {
        Self { tasks }
    }
}

impl Exec for AndTask {
    fn run(&self, controller: &Controller) -> Result<(), String> {
        self.tasks
            .iter()
            .map(|task| task.run(controller))
            .collect()
    }
}

// 若任何 Match 失败则失败
// 成功返回所有匹配 Rect
#[derive(Debug)]
pub struct OrTask {
    tasks: Vec<Box<dyn Exec>>,
}

impl OrTask {
    pub fn new(tasks: Vec<Box<dyn Exec>>) -> Self {
        Self { tasks }
    }
}

impl Exec for OrTask {
    fn run(&self, controller: &Controller) -> Result<(), String> {
        for task in &self.tasks {
            if task.run(controller).is_ok() {
                return Ok(());
            }
        }
        Err("match failed".to_string())
    }
}
