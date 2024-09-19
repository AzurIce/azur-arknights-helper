pub mod click;
pub mod click_match;
pub mod press_esc;
pub mod press_home;
pub mod swipe;

use aah_resource::manifest::Action;
pub use click::Click;
pub use click_match::ClickMatch;
pub use press_esc::PressEsc;
pub use press_home::PressHome;
pub use swipe::Swipe;

use super::{navigate::Navigate, Runnable};
use crate::AAH;

use std::time::Duration;

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
