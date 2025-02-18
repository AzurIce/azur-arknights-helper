//! 此模块包含对任务的定义
//!
//! 每一个任务对应一个 `resources/tasks` 中的 `xxx.toml` 文件，其中内容对应 [`Task`] 结构。
//!
//! 每一个 [`Task`] 由若干 [`TaskStep`] 组成，每一个 [`TaskStep`] 包含一个 [`Action`] 和一些额外的属性。
//!
//! [`Action`]` 即为 [`super::Task`] 中每一个 [`super::TaskStep`] 中的实际操作。[`Action`] 本身只是对操作的数据表示，实际的实现在 [`Runnable`] 中。
//!
//!

use std::{fmt::Debug, time::Duration};

use color_print::cprintln;
use image::DynamicImage;
use log::info;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{resource::GetTask, Core, TaskRecipe};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByNameAction {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ByNameActionSet {
    ByName(ByNameAction),
}

/// Action are the tasks you can use in the configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Action<ActionSet: Debug + Clone> {
    /// Run a task referenced by the name
    ByName(ByNameActionSet),
    Detailed(ActionSet),
}

impl<ActionSet: Debug + Clone> Action<ActionSet> {
    pub fn by_name(name: impl AsRef<str>) -> Self {
        let name = name.as_ref().to_string();
        // 这一坨莫名其妙的一层包一层其实是为了统一 toml 格式（）
        Self::ByName(ByNameActionSet::ByName(ByNameAction { name }))
    }
    pub fn detailed(action: impl Into<ActionSet>) -> Self {
        Self::Detailed(action.into())
    }
}

impl<C, R, T, ActionSet> TaskRecipe<T> for Action<ActionSet>
where
    R: GetTask<ActionSet>,
    T: Core<Controller = C, Resource = R>,
    ActionSet: TaskRecipe<T> + Debug + Clone,
{
    type Res = ();
    fn run(&self, runner: &T) -> anyhow::Result<Self::Res> {
        match self {
            Action::ByName(ByNameActionSet::ByName(ByNameAction { name })) => {
                let task = runner
                    .resource()
                    .get_task(name)
                    .ok_or_else(|| anyhow::anyhow!("failed to get task by name: {:?}", name))?;
                task.run(runner).map(|_| ())
            }
            Action::Detailed(action) => action.run(runner).map(|_| ()),
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
/// 一个完整的 [`Task`] 由若干 [`TaskStep`] 组成
pub struct Task<ActionSet: Debug + Clone> {
    /// Task 的名称
    pub name: String,
    /// Task 的描述
    pub desc: Option<String>,
    /// Task 的步骤
    pub steps: Vec<TaskStep<ActionSet>>,
}

impl<ActionSet: Debug + Clone> Task<ActionSet> {
    pub fn from_steps(steps: Vec<TaskStep<ActionSet>>) -> Self {
        Self {
            name: "unnamed".to_string(),
            desc: None,
            steps,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.to_string());
        self
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
/// TaskStep 是对 [`Action`] 的封装，可以设置一些额外的属性
pub struct TaskStep<ActionSet: Debug + Clone> {
    /// 在此 Step 开始前的延迟
    pub delay_sec: Option<f32>,
    /// 如果此 Step 失败，是否跳过（否则会直接中断退出）
    pub skip_if_failed: Option<bool>,
    /// 重复次数
    pub repeat: Option<u32>,
    /// 每次重试次数
    pub retry: Option<i32>,
    /// 在此 Step 中要执行的 Action
    pub action: Action<ActionSet>,
}

impl<ActionSet: Debug + Clone> TaskStep<ActionSet> {
    pub fn from_action(action: Action<ActionSet>) -> Self {
        Self {
            delay_sec: None,
            skip_if_failed: None,
            repeat: None,
            retry: None,
            action,
        }
    }

    pub fn with_delay(mut self, sec: f32) -> Self {
        self.delay_sec = Some(sec);
        self
    }

    pub fn skip_if_failed(mut self) -> Self {
        self.skip_if_failed = Some(true);
        self
    }

    pub fn with_repeat(mut self, times: u32) -> Self {
        self.repeat = Some(times);
        self
    }

    pub fn with_retry(mut self, times: i32) -> Self {
        self.retry = Some(times);
        self
    }
}

// /// 任务事件
// ///
// /// - `Log(String)`: log 信息
// /// - `Img(DynamicImage)`: 标记过的图片
// #[derive(Clone)]
// #[non_exhaustive]
// pub enum TaskEvt<T: Debug + Clone> {
//     ExecStat {
//         step: TaskStep<T>,
//         cur: usize,
//         total: usize,
//     },
//     MatchTaskRes {},
//     Log(String),
//     AnnotatedImg(DynamicImage),
//     BattleAnalyzerRes(BattleAnalyzerOutput),
// }

// impl<T: Debug + Clone> Debug for TaskEvt<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             TaskEvt::ExecStat { step, cur, total } => {
//                 write!(f, "TaskEvt::ExecStat({:?}, {}/{})", step, cur, total)
//             }
//             TaskEvt::Log(log) => write!(f, "TaskEvt::Log({})", log),
//             TaskEvt::AnnotatedImg(_img) => write!(f, "TaskEvt::AnnotatedImg"),
//             TaskEvt::BattleAnalyzerRes(res) => write!(f, "TaskEvt::BattleAnalyzerRes({:?})", res),
//             TaskEvt::MatchTaskRes { .. } => write!(f, "TaskEvt::MatchTaskRes"),
//         }
//     }
// }

impl<C, R, T, ActionSet> TaskRecipe<T> for Task<ActionSet>
where
    R: GetTask<ActionSet>,
    T: Core<Controller = C, Resource = R>,
    ActionSet: TaskRecipe<T> + Debug + Clone,
{
    type Res = ();
    fn run(&self, runner: &T) -> anyhow::Result<Self::Res> {
        info!("[Task<{}>] running...", self.name);
        for (i, step) in self.steps.iter().enumerate() {
            info!(
                "[Task<{}>] running step {}/{}: {:?}",
                self.name,
                i,
                self.steps.len(),
                step
            );
            // runner.emit_task_evt(TaskEvt::ExecStat {
            //     step: step.clone(),
            //     cur: i,
            //     total: self.steps.len(),
            // });
            cprintln!(
                "<m><strong>[Task]</strong></m>: executing task {}({}/{}): {:?}",
                self.name,
                i,
                self.steps.len(),
                step
            );
            let res = step.run(runner);
            if res.is_err() && !step.skip_if_failed.unwrap_or(false) {
                return res;
            }
        }
        Ok(())
    }
}

impl<C, R, T, ActionSet> TaskRecipe<T> for TaskStep<ActionSet>
where
    R: GetTask<ActionSet>,
    T: Core<Controller = C, Resource = R>,
    ActionSet: TaskRecipe<T> + Debug + Clone,
{
    type Res = ();
    fn run(&self, runner: &T) -> anyhow::Result<Self::Res> {
        std::thread::sleep(Duration::from_secs_f32(self.delay_sec.unwrap_or(0.0)));

        let exec = || {
            let mut res = self.action.run(runner);
            // debug!("TaskStep::run: {:?}", res);
            match self.retry {
                None => return res,
                Some(retry) => {
                    if retry < 0 {
                        while res.is_err() {
                            res = self.action.run(runner);
                        }
                    } else {
                        for _ in 0..self.retry.unwrap_or(0) {
                            if res.is_ok() {
                                break;
                            }
                            res = self.action.run(runner);
                        }
                    }
                }
            }
            res
        };

        // 先执行一次
        let mut res = exec();
        for _ in 0..self.repeat.unwrap_or(0) {
            // Fail fast for repeat
            if res.is_err() {
                break;
            }
            res = exec()
        }
        res
    }
}

#[cfg(test)]
mod test {
    use crate::android::{
        self,
        actions::{ClickMatchTemplate, Press},
    };

    use super::*;

    #[test]
    fn test_serde_action() {
        let action = Action::<android::actions::ActionSet>::by_name("test".to_string());
        let toml = toml::to_string_pretty(&action).unwrap();
        println!("{toml}");

        let action = Action::<android::actions::ActionSet>::detailed(Press::esc());
        let toml = toml::to_string_pretty(&action).unwrap();
        println!("{toml}");

        let action = Action::<android::actions::ActionSet>::detailed(ClickMatchTemplate::new(
            "template.png",
        ));
        let toml = toml::to_string_pretty(&action).unwrap();
        println!("{toml}");
    }

    #[test]
    fn test_serde_task() {
        let task = Task::<android::actions::ActionSet> {
            name: "test".to_string(),
            desc: Some("test".to_string()),
            steps: vec![
                TaskStep {
                    delay_sec: Some(1.0),
                    skip_if_failed: Some(true),
                    repeat: Some(2),
                    retry: Some(3),
                    action: Action::by_name("test"),
                },
                TaskStep {
                    delay_sec: Some(1.0),
                    skip_if_failed: Some(true),
                    repeat: Some(2),
                    retry: Some(3),
                    action: Action::detailed(Press::esc()),
                },
            ],
        };
        let toml = toml::to_string_pretty(&task).unwrap();
        println!("{toml}");
    }
}
