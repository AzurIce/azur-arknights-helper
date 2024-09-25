pub mod click;
pub mod click_match_template;
pub mod press_esc;
pub mod press_home;
pub mod swipe;

pub use click::Click;
pub use click_match_template::ClickMatchTemplate;
pub use press_esc::PressEsc;
pub use press_home::PressHome;
use serde::{Deserialize, Serialize};
pub use swipe::Swipe;

use super::{navigate::Navigate, Runnable};
use crate::AAH;

/// Action is the basic executable unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    ByName(String),
    ActionPressEsc(PressEsc),
    ActionPressHome(PressHome),
    ActionClick(Click),
    ActionSwipe(Swipe),
    ActionClickMatchTemplate(ClickMatchTemplate),
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
            Action::ActionPressEsc(action) => action.run(aah),
            Action::ActionPressHome(action) => action.run(aah),
            Action::ActionClick(action) => action.run(aah),
            Action::ActionSwipe(action) => action.run(aah),
            Action::ActionClickMatchTemplate(action) => action
                .run(aah)
                .map_err(|err| format!("{err}, caused by: {}", err.root_cause())),
            Action::NavigateIn(navigate) => Navigate::NavigateIn(navigate.clone()).run(aah),
            Action::NavigateOut(navigate) => Navigate::NavigateOut(navigate.clone()).run(aah),
        }
    }
}
