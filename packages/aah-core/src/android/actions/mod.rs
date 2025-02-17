pub mod click;
pub mod click_match_template;
pub mod press;
pub mod swipe;

use std::time::Duration;

use aah_controller::Controller;
pub use click::Click;
pub use click_match_template::ClickMatchTemplate;
pub use press::Press;
use serde::{Deserialize, Serialize};
pub use swipe::Swipe;

use crate::{resource::ResRoot, Core, TaskRecipe};

/// Action are the tasks you can use in the configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionSet {
    /// [`Press`]
    Press(Press),
    /// [`Click`]
    Click(Click),
    /// [`Swipe`]
    Swipe(Swipe),
    /// [`ClickMatchTemplate`]
    ClickMatchTemplate(ClickMatchTemplate),
}

impl ActionSet {
    pub fn press_esc() -> Self {
        Self::Press(Press::esc())
    }
    pub fn press_home() -> Self {
        Self::Press(Press::home())
    }
    pub fn click(x: u32, y: u32) -> Self {
        Self::Click(Click::new(x, y))
    }
    pub fn swipe(
        start: (u32, u32),
        end: (i32, i32),
        duration: Duration,
        slope_in: f32,
        slope_out: f32,
    ) -> Self {
        Self::Swipe(Swipe::new(start, end, duration, slope_in, slope_out))
    }
    pub fn click_match_template(template: impl AsRef<str>) -> Self {
        Self::ClickMatchTemplate(ClickMatchTemplate::new(template))
    }
}

impl<T, C, R> TaskRecipe<T> for ActionSet
where
    C: Controller,
    R: ResRoot,
    T: Core<Controller = C, Resource = R>,
{
    type Res = ();
    fn run(&self, aah: &T) -> anyhow::Result<Self::Res> {
        match self {
            ActionSet::Press(action) => action.run(aah),
            ActionSet::Click(action) => action.run(aah),
            ActionSet::Swipe(action) => action.run(aah),
            ActionSet::ClickMatchTemplate(action) => action.run(aah),
        }
    }
}
