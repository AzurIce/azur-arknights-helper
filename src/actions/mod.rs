use std::time::Duration;
use serde::{Deserialize, Serialize};

use super::AahCore;

pub mod battle;
pub mod choose_level;
pub mod copilot;

use auto_play::actions::{Click, ClickMatchTemplate, Press, Swipe, Task, Runnable};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Action {
    /// [`Press`]
    Press(Press),
    /// [`Click`]
    Click(Click),
    /// [`Swipe`]
    Swipe(Swipe),
    /// [`ClickMatchTemplate`]
    ClickMatchTemplate(ClickMatchTemplate),
    Task(Task<Action>),
    // BattleDeploy(battle::Deploy),
    // BattleRetreat(battle::Retreat),
    // BattleUseSkill(battle::UseSkill),
    // Copilot(copilot::Copilot),
    // ChooseLevel(choose_level::ChooseLevel),
}

impl Action {
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
    pub fn task(name: impl AsRef<str>) -> Self {
        Self::Task(Task::new(name))
    }
}

impl Runnable<AahCore> for Action {
    type Output = ();
    fn execute(&self, executor: &AahCore) -> anyhow::Result<Self::Output> {
        match self {
            Action::Press(press) => press.execute(executor),
            Action::Click(click) => click.execute(executor),
            Action::Swipe(swipe) => swipe.execute(executor),
            Action::ClickMatchTemplate(click_match_template) => click_match_template.execute(executor),
            Action::Task(task) => task.execute(executor),
        }
    }
}
