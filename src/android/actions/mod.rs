pub mod click;
pub mod click_match_template;
pub mod press;
pub mod swipe;

use aah_controller::Controller;
pub use click::Click;
pub use click_match_template::ClickMatchTemplate;
pub use press::Press;
use serde::{Deserialize, Serialize};
pub use swipe::Swipe;

use crate::{task::Runnable, CachedScreenCapper, GetTemplate};

/// Action are the tasks you can use in the configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AndroidActionSet {
    /// [`Press`]
    Press(Press),
    /// [`Click`]
    Click(Click),
    /// [`Swipe`]
    Swipe(Swipe),
    /// [`ClickMatchTemplate`]
    ClickMatchTemplate(ClickMatchTemplate),
}

impl<T: Controller + CachedScreenCapper + GetTemplate> Runnable<T> for AndroidActionSet {
    type Res = ();
    fn run(&self, runner: &T) -> anyhow::Result<Self::Res> {
        match self {
            AndroidActionSet::Press(action) => action.run(runner),
            AndroidActionSet::Click(action) => action.run(runner),
            AndroidActionSet::Swipe(action) => action.run(runner),
            AndroidActionSet::ClickMatchTemplate(action) => action.run(runner),
        }
    }
}
