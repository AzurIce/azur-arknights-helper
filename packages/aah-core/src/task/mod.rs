use crate::controller::Controller;

pub mod builtins;
pub mod match_task;
pub mod wrapper;

pub trait Task {
    type Res = ();
    type Err = ();
    fn run(&self, controller: &Controller) -> Result<Self::Res, Self::Err>;
}

/// 任务 Trait
pub trait Exec: std::fmt::Debug {
    fn run(&self, controller: &Controller) -> Result<(), String>;
}

/// 带返回结果的任务 Trait
pub trait ExecResult: std::fmt::Debug {
    type Type = ();
    fn result(&self, controller: &Controller) -> Result<Self::Type, String>;
}

impl<T: ExecResult> Exec for T {
    fn run(&self, controller: &Controller) -> Result<(), String> {
        self.result(controller).map(|_| ())
    }
}
