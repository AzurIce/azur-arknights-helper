use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::Duration;

/// A Trait for generic pre/post process for a task
pub trait TaskWrapper: Default + Debug + Serialize {
    fn run<T, E>(&self, run: impl Fn() -> Result<T, E>) -> Result<T, E> {
        run()
    }
}

/// A Generic TaskWrapper
/// - `delay`: secs to wait before executing the task
/// - `retry`: max retry times when task is failed
/// - `repeat`: repeat times (each repeat will have above retry times)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenericTaskWrapper {
    #[serde(default)]
    pub delay: f32,
    #[serde(default)]
    pub retry: usize,
    #[serde(default)]
    pub repeat: usize,
}

impl Default for GenericTaskWrapper {
    fn default() -> Self {
        Self {
            delay: 0.0,
            retry: 0,
            repeat: 1,
        }
    }
}

impl TaskWrapper for GenericTaskWrapper {
    fn run<T, E>(&self, run: impl Fn() -> Result<T, E>) -> Result<T, E> {
        std::thread::sleep(Duration::from_secs_f32(self.delay));

        let exec = || {
            let mut res = run();
            for _ in 0..self.retry {
                // Success fast for retry
                if res.is_ok() {
                    break;
                }
                res = run();
            }
            res
        };

        let mut res = exec();
        if self.repeat != 0 {
            for _ in 0..self.repeat - 1 {
                // Fail fast for repeat
                if res.is_err() {
                    break;
                }
                res = exec()
            }
        }
        res
    }
}
