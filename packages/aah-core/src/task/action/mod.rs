pub mod click;
pub mod click_match_template;
#[deprecated(note = "Use ByName Action instead")]
pub mod navigate;
pub mod press_esc;
pub mod press_home;
pub mod swipe;

pub use click::Click;
pub use click_match_template::ClickMatchTemplate;
pub use navigate::{NavigateIn, NavigateOut};
pub use press_esc::PressEsc;
pub use press_home::PressHome;
use serde::{Deserialize, Serialize};
pub use swipe::Swipe;

use super::Runnable;
use crate::AAH;

/// Action are the tasks you can use in the configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Run a task referenced by the name
    ByName(String),
    /// [`PressEsc`]`
    PressEsc(PressEsc),
    /// [`PressHome`]
    PressHome(PressHome),
    /// [`Click`]
    Click(Click),
    /// [`Swipe`]
    Swipe(Swipe),
    /// [`ClickMatchTemplate`]
    ClickMatchTemplate(ClickMatchTemplate),
    // Navigate
    #[deprecated(note = "Use ByName Action instead")]
    NavigateIn(NavigateIn),
    #[deprecated(note = "Use ByName Action instead")]
    NavigateOut(NavigateOut),
}

impl Runnable for Action {
    type Res = ();
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        match self {
            Action::ByName(name) => aah.run_task(name),
            // BuiltinTask::Multi(task) => task.run(aah),
            Action::PressEsc(action) => action.run(aah),
            Action::PressHome(action) => action.run(aah),
            Action::Click(action) => action.run(aah),
            Action::Swipe(action) => action.run(aah),
            Action::ClickMatchTemplate(action) => action
                .run(aah)
                .map_err(|err| format!("{err}, caused by: {}", err.root_cause())),
            Action::NavigateIn(action) => action.run(aah),
            Action::NavigateOut(action) => action.run(aah),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serde_action() {
        let action = Action::PressEsc(PressEsc);
        let toml = toml::to_string_pretty(&action).unwrap();
        println!("{toml}")
    }
}
