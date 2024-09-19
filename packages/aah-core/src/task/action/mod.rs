pub mod click_match;
pub mod click;
pub mod press_esc;
pub mod press_home;
pub mod swipe;

pub use click_match::ClickMatch;
pub use click::Click;
pub use press_esc::PressEsc;
pub use press_home::PressHome;
pub use swipe::Swipe;

use crate::AAH;
use super::{match_task::MatchTask, navigate::Navigate, Runnable};

use serde::{Deserialize, Serialize};
use std::time::Duration;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    ByName(String),
    // Multi(Multi),
    // Action
    ActionPressEsc,
    ActionPressHome,
    ActionClick {
        x: u32,
        y: u32,
    },
    ActionSwipe {
        p1: (u32, u32),
        p2: (i32, i32),
        duration: f32,
        slope_in: f32,
        slope_out: f32,
    },
    ActionClickMatch {
        #[serde(flatten)]
        match_task: MatchTask,
    },
    // Navigate
    NavigateIn(String),
    NavigateOut(String),
}

impl Runnable for Action {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        match self {
            Action::ByName(name) => aah.run_task(name),
            // BuiltinTask::Multi(task) => task.run(aah),
            Action::ActionPressEsc => PressEsc.run(aah),
            Action::ActionPressHome => PressHome.run(aah),
            Action::ActionClick { x, y } => Click::new(*x, *y).run(aah),
            Action::ActionSwipe {
                p1,
                p2,
                duration,
                slope_in,
                slope_out,
            } => Swipe::new(
                *p1,
                *p2,
                Duration::from_secs_f32(*duration),
                *slope_in,
                *slope_out,
            )
            .run(aah),
            Action::ActionClickMatch { match_task } => ClickMatch::new(match_task.clone()).run(aah),
            Action::NavigateIn(navigate) => Navigate::NavigateIn(navigate.clone()).run(aah),
            Action::NavigateOut(navigate) => Navigate::NavigateOut(navigate.clone()).run(aah),
        }
    }
}
