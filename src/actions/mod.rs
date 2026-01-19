use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::AahCore;
// Removed trait imports: HasController, GetTemplate

pub mod battle;
pub mod choose_level;
pub mod copilot;

pub trait Runnable<Context> {
    type Output;
    fn execute(&self, context: &Context) -> anyhow::Result<Self::Output>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Click {
    pub x: u32,
    pub y: u32,
}

impl Click {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

// Implemented specifically for AahCore
impl Runnable<AahCore> for Click {
    type Output = ();
    fn execute(&self, context: &AahCore) -> anyhow::Result<Self::Output> {
        context
            .controller()
            .click(self.x, self.y)
            .map_err(|e| anyhow::anyhow!(e))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Key {
    Home,
    Escape,
    // Add others
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Press {
    pub key: Key,
}

impl Press {
    pub fn esc() -> Self {
        Self { key: Key::Escape }
    }
    pub fn home() -> Self {
        Self { key: Key::Home }
    }
}

impl Runnable<AahCore> for Press {
    type Output = ();
    fn execute(&self, _context: &AahCore) -> anyhow::Result<Self::Output> {
        // TODO: Fix Key type mismatch.
        Err(anyhow::anyhow!(
            "Press action not fully implemented: Key type mismatch"
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swipe {
    pub start: (u32, u32),
    pub end: (i32, i32),
    pub duration: Duration,
    pub slope_in: f32,
    pub slope_out: f32,
}

impl Swipe {
    pub fn new(
        start: (u32, u32),
        end: (i32, i32),
        duration: Duration,
        slope_in: f32,
        slope_out: f32,
    ) -> Self {
        Self {
            start,
            end,
            duration,
            slope_in,
            slope_out,
        }
    }
}

impl Runnable<AahCore> for Swipe {
    type Output = ();
    fn execute(&self, context: &AahCore) -> anyhow::Result<Self::Output> {
        context
            .controller()
            .swipe(
                (self.start.0, self.start.1),
                (self.end.0 as i32, self.end.1 as i32),
                self.duration,
                self.slope_in,
                self.slope_out,
            )
            .map_err(|e| anyhow::anyhow!(e))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickMatchTemplate {
    pub template: String,
}

impl ClickMatchTemplate {
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }
}

impl Runnable<AahCore> for ClickMatchTemplate {
    type Output = ();
    fn execute(&self, context: &AahCore) -> anyhow::Result<Self::Output> {
        let _screen = context.screen_cache_or_cap()?;
        let _template_img = context.get_template(&self.template)?;
        // TODO: Implement matching
        Err(anyhow::anyhow!("ClickMatchTemplate not implemented"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep<A> {
    pub action: A,
    pub delay_after: Option<Duration>,
}

impl<A> TaskStep<A> {
    pub fn from_action(action: A) -> Self {
        Self {
            action,
            delay_after: None,
        }
    }

    pub fn with_delay(mut self, secs: f32) -> Self {
        self.delay_after = Some(Duration::from_secs_f32(secs));
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task<A> {
    pub name: String,
    pub steps: Vec<TaskStep<A>>,
}

impl<A> Task<A> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
        }
    }

    pub fn from_steps(steps: Vec<TaskStep<A>>) -> Self {
        Self {
            name: "Task".to_string(),
            steps,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

impl<A: Runnable<Context>, Context> Runnable<Context> for Task<A> {
    type Output = ();
    fn execute(&self, context: &Context) -> anyhow::Result<Self::Output> {
        for step in &self.steps {
            step.action.execute(context)?;
            if let Some(delay) = step.delay_after {
                std::thread::sleep(delay);
            }
        }
        Ok(())
    }
}

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
        Self::ClickMatchTemplate(ClickMatchTemplate::new(template.as_ref().to_string()))
    }
    pub fn task(name: impl AsRef<str>) -> Self {
        Self::Task(Task::new(name.as_ref().to_string()))
    }
}

impl Runnable<AahCore> for Action {
    type Output = ();
    fn execute(&self, executor: &AahCore) -> anyhow::Result<Self::Output> {
        match self {
            Action::Press(press) => press.execute(executor),
            Action::Click(click) => click.execute(executor),
            Action::Swipe(swipe) => swipe.execute(executor),
            Action::ClickMatchTemplate(click_match_template) => {
                click_match_template.execute(executor)
            }
            Action::Task(task) => task.execute(executor),
        }
    }
}
