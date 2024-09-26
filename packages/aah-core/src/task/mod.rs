use std::{fmt::Debug, time::Duration};

use action::Action;
use color_print::cprintln;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{vision::analyzer::battle::BattleAnalyzerOutput, AAH};

pub mod action;
pub mod battle;
pub mod choose_level;
pub mod copilot;

pub trait Runnable {
    type Res = ();
    type Err = ();
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err>;
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    /// Task 的名称
    pub name: String,
    /// Task 的描述
    pub desc: Option<String>,
    /// Task 的步骤
    pub steps: Vec<TaskStep>,
}

impl Task {
    pub fn from_steps(steps: Vec<TaskStep>) -> Self {
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
pub struct TaskStep {
    /// 在此 Step 开始前的延迟
    pub delay_sec: Option<f32>,
    /// 如果此 Step 失败，是否跳过（否则会直接中断退出）
    pub skip_if_failed: Option<bool>,
    /// 重复次数
    pub repeat: Option<u32>,
    /// 每次重试次数
    pub retry: Option<i32>,
    /// 在此 Step 中要执行的 Action
    pub action: Action,
}

impl TaskStep {
    pub fn action(task: impl Into<Action>) -> Self {
        let task = task.into();
        Self {
            delay_sec: None,
            skip_if_failed: None,
            repeat: None,
            retry: None,
            action: task,
        }
    }

    pub fn deplay_sec_f32(mut self, sec: f32) -> Self {
        self.delay_sec = Some(sec);
        self
    }

    pub fn skip_if_failed(mut self) -> Self {
        self.skip_if_failed = Some(true);
        self
    }

    pub fn repeat(mut self, times: u32) -> Self {
        self.repeat = Some(times);
        self
    }

    pub fn retry(mut self, times: i32) -> Self {
        self.retry = Some(times);
        self
    }
}

/// 任务事件
///
/// - `Log(String)`: log 信息
/// - `Img(DynamicImage)`: 标记过的图片
#[derive(Clone)]
#[non_exhaustive]
pub enum TaskEvt {
    ExecStat {
        step: TaskStep,
        cur: usize,
        total: usize,
    },
    MatchTaskRes {},
    Log(String),
    AnnotatedImg(DynamicImage),
    BattleAnalyzerRes(BattleAnalyzerOutput),
}

impl Debug for TaskEvt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskEvt::ExecStat { step, cur, total } => {
                write!(f, "TaskEvt::ExecStat({:?}, {}/{})", step, cur, total)
            }
            TaskEvt::Log(log) => write!(f, "TaskEvt::Log({})", log),
            TaskEvt::AnnotatedImg(_img) => write!(f, "TaskEvt::AnnotatedImg"),
            TaskEvt::BattleAnalyzerRes(res) => write!(f, "TaskEvt::BattleAnalyzerRes({:?})", res),
            TaskEvt::MatchTaskRes { .. } => write!(f, "TaskEvt::MatchTaskRes"),
        }
    }
}

impl Runnable for Task {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        for (i, step) in self.steps.iter().enumerate() {
            aah.emit_task_evt(TaskEvt::ExecStat {
                step: step.clone(),
                cur: i,
                total: self.steps.len(),
            });
            cprintln!(
                "<m><strong>[Task]</strong></m>: executing task {}({}/{}): {:?}",
                self.name,
                i,
                self.steps.len(),
                step
            );
            let res = step.run(aah);
            if res.is_err() && !step.skip_if_failed.unwrap_or(false) {
                return res;
            }
        }
        Ok(())
    }
}

impl Runnable for TaskStep {
    type Res = ();
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        std::thread::sleep(Duration::from_secs_f32(self.delay_sec.unwrap_or(0.0)));

        let exec = || {
            let mut res = self.action.run(aah);
            match self.retry {
                None => return res,
                Some(retry) => {
                    if retry < 0 {
                        while res.is_err() {
                            res = self.action.run(aah);
                        }
                    } else {
                        for _ in 0..self.retry.unwrap_or(0) {
                            if res.is_ok() {
                                break;
                            }
                            res = self.action.run(aah);
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
