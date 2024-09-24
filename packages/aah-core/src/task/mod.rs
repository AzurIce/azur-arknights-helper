use std::{fmt::Debug, time::Duration};

use aah_resource::manifest::task::{Task, TaskStep};
use color_print::cprintln;
use image::DynamicImage;

use crate::{vision::analyzer::battle::BattleAnalyzerOutput, AAH};

pub mod action;
pub mod battle;
pub mod match_task;
pub mod navigate;

pub trait Runnable {
    type Res = ();
    type Err = ();
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err>;
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
    MatchTaskRes {

    },
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
            TaskEvt::MatchTaskRes { .. } => write!(f, "TaskEvt::MatchTaskRes")
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
