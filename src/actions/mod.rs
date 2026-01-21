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

// Duration serialization module for TOML format (delay_sec = f32)
mod duration_secs_f32_option {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(d) => serializer.serialize_some(&d.as_secs_f32()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs: Option<f32> = Option::deserialize(deserializer)?;
        Ok(secs.map(Duration::from_secs_f32))
    }
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
pub struct ByName {
    pub name: String,
}

impl ByName {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Runnable<AahCore> for ByName {
    type Output = ();
    fn execute(&self, context: &AahCore) -> anyhow::Result<Self::Output> {
        let task = context
            .get_task(&self.name)
            .ok_or_else(|| anyhow::anyhow!("Task '{}' not found", self.name))?;
        task.execute(context)
    }
}

/// Options for controlling action execution behavior
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionOptions {
    /// Retry count: -1 = infinite retry, 0 = no retry, n > 0 = n+1 attempts
    #[serde(default)]
    pub retry: i32,

    /// If true, skip on failure instead of returning error
    #[serde(default)]
    pub skip_if_failed: bool,

    /// Delay to wait after action execution
    #[serde(default)]
    #[serde(rename = "delay_sec")]
    #[serde(with = "duration_secs_f32_option")]
    pub delay_after: Option<Duration>,
}

impl ActionOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_retry(mut self, retry: i32) -> Self {
        self.retry = retry;
        self
    }

    pub fn with_skip_if_failed(mut self, skip: bool) -> Self {
        self.skip_if_failed = skip;
        self
    }

    pub fn with_delay(mut self, secs: f32) -> Self {
        self.delay_after = Some(Duration::from_secs_f32(secs));
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep<T> {
    pub action: T,

    #[serde(flatten)]
    pub options: ActionOptions,
}

impl<T> TaskStep<T> {
    pub fn from_action(action: T) -> Self {
        Self {
            action,
            options: ActionOptions::default(),
        }
    }

    pub fn with_delay(mut self, secs: f32) -> Self {
        self.options = self.options.with_delay(secs);
        self
    }

    pub fn with_retry(mut self, retry: i32) -> Self {
        self.options = self.options.with_retry(retry);
        self
    }

    pub fn with_skip_if_failed(mut self, skip: bool) -> Self {
        self.options = self.options.with_skip_if_failed(skip);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task<T> {
    pub name: String,

    #[serde(default)]
    pub desc: Option<String>,

    pub steps: Vec<TaskStep<T>>,
}

impl<A> Task<A> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            desc: None,
            steps: Vec::new(),
        }
    }

    pub fn from_steps(steps: Vec<TaskStep<A>>) -> Self {
        Self {
            name: "Task".to_string(),
            desc: None,
            steps,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_desc(mut self, desc: impl Into<String>) -> Self {
        self.desc = Some(desc.into());
        self
    }
}

impl<A: Runnable<Context>, Context> Runnable<Context> for Task<A> {
    type Output = ();
    fn execute(&self, context: &Context) -> anyhow::Result<Self::Output> {
        for (idx, step) in self.steps.iter().enumerate() {
            let opts = &step.options;
            let mut attempts = 0;
            let max_attempts = if opts.retry < 0 {
                i32::MAX
            } else {
                opts.retry + 1
            };

            let mut last_error = None;

            while attempts < max_attempts {
                match step.action.execute(context) {
                    Ok(_) => {
                        // Success, break out of retry loop
                        break;
                    }
                    Err(e) => {
                        last_error = Some(e);
                        attempts += 1;

                        if attempts >= max_attempts {
                            // Reached max retry attempts
                            if opts.skip_if_failed {
                                eprintln!(
                                    "Step {} failed after {} attempts, skipping: {:?}",
                                    idx, attempts, last_error
                                );
                                break;
                            } else {
                                return Err(last_error.unwrap());
                            }
                        }

                        // Wait a short time before retrying
                        if attempts < max_attempts {
                            std::thread::sleep(Duration::from_millis(100));
                        }
                    }
                }
            }

            // Execute delay after step
            if let Some(delay) = opts.delay_after {
                std::thread::sleep(delay);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// [`Click`]
    Click(Click),
    /// [`Press`]
    Press(Press),
    /// [`Swipe`]
    Swipe(Swipe),
    /// [`ClickMatchTemplate`]
    ClickMatchTemplate(ClickMatchTemplate),
    /// [`ByName`]
    ByName(ByName),
    // /// [`Task`]
    // Task(Task<Action>),
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
    pub fn by_name(name: impl Into<String>) -> Self {
        Self::ByName(ByName::new(name))
    }
    // pub fn task(name: impl AsRef<str>) -> Self {
    //     Self::Task(Task::new(name.as_ref().to_string()))
    // }
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
            Action::ByName(by_name) => by_name.execute(executor),
            // Action::Task(task) => task.execute(executor),
        }
    }
}
