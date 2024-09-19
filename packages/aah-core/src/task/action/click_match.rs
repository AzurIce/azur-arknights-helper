use aah_resource::manifest::MatchTask;

use crate::{task::Runnable, AAH};

pub struct ClickMatch {
    match_task: MatchTask,
}

impl ClickMatch {
    pub fn new(match_task: MatchTask) -> Self {
        Self { match_task }
    }
}

impl Runnable for ClickMatch {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        aah.controller
            .click_in_rect(self.match_task.run(&aah)?)
            .map_err(|err| format!("controller error: {:?}", err))
    }
}
