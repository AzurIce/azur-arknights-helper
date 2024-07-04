mod action_click;
mod action_click_match;
mod action_press_esc;
mod action_press_home;
mod action_swipe;
mod by_name;

mod multi;
mod navigate;

pub use action_click::ActionClick;
pub use action_click_match::ActionClickMatch;
pub use action_press_esc::ActionPressEsc;
pub use action_press_home::ActionPressHome;
pub use action_swipe::ActionSwipe;
pub use by_name::ByName;
pub use multi::Multi;
pub use navigate::Navigate;
use serde::{Deserialize, Serialize};

use crate::{
    task::{match_task::MatchTask, wrapper::GenericTaskWrapper},
    AAH,
};

use super::{Task, TaskEvt};

pub fn test_tasks() -> Vec<(&'static str, BuiltinTask)> {
    vec![
        (
            "press_esc",
            BuiltinTask::ActionPressEsc(ActionPressEsc::new(None)),
        ),
        (
            "press_home",
            BuiltinTask::ActionPressHome(ActionPressHome::new(None)),
        ),
        (
            "click",
            BuiltinTask::ActionClick(ActionClick::new(0, 0, Some(GenericTaskWrapper::default()))),
        ),
        (
            "swipe",
            BuiltinTask::ActionSwipe(ActionSwipe::new((0, 0), (200, 0), 1.0, None)),
        ),
        (
            "click_match",
            BuiltinTask::ActionClickMatch(ActionClickMatch::new(
                MatchTask::Template("ButtonToggleTopNavigator.png".to_string()),
                None,
            )),
        ),
        ("navigate_in", BuiltinTask::NavigateIn("name".to_string())),
        ("navigate_out", BuiltinTask::NavigateIn("name".to_string())),
        (
            "by_name",
            BuiltinTask::ByName(ByName::new(
                "press_esc",
                Some(GenericTaskWrapper::default()),
            )),
        ),
        (
            "multiple",
            BuiltinTask::Multi(Multi::new(
                vec![
                    BuiltinTask::ActionPressEsc(ActionPressEsc::new(None)),
                    BuiltinTask::ActionPressHome(ActionPressHome::new(None)),
                ],
                false,
                None,
            )),
        ),
    ]
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BuiltinTask {
    ByName(ByName),
    Multi(Multi),
    // Action
    ActionPressEsc(ActionPressEsc),
    ActionPressHome(ActionPressHome),
    ActionClick(ActionClick),
    ActionSwipe(ActionSwipe),
    ActionClickMatch(ActionClickMatch),
    // Navigate
    NavigateIn(String),
    NavigateOut(String),
}

impl Task for BuiltinTask {
    type Err = String;
    fn run(&self, aah: &AAH, on_task_evt: impl Fn(TaskEvt)) -> Result<Self::Res, Self::Err> {
        match self {
            BuiltinTask::ByName(task) => task.run(aah, on_task_evt),
            BuiltinTask::Multi(task) => task.run(aah, on_task_evt),
            BuiltinTask::ActionPressEsc(task) => task.run(aah, on_task_evt),
            BuiltinTask::ActionPressHome(task) => task.run(aah, on_task_evt),
            BuiltinTask::ActionClick(task) => task.run(aah, on_task_evt),
            BuiltinTask::ActionSwipe(task) => task.run(aah, on_task_evt),
            BuiltinTask::ActionClickMatch(task) => task.run(aah, on_task_evt),
            BuiltinTask::NavigateIn(navigate) => Navigate::NavigateIn(navigate.clone()).run(aah, on_task_evt),
            BuiltinTask::NavigateOut(navigate) => Navigate::NavigateOut(navigate.clone()).run(aah, on_task_evt),
        }
    }
}
