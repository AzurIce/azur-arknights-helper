use std::{path::Path, time::Duration};

use image::math::Rect;
use imageproc::point::Point;

use crate::{
    controller::Controller,
    vision::matcher::{MatchType, Matcher},
};

mod auto_recruit;
mod navigate;

#[cfg(test)]
mod test {
    use std::error::Error;

    use super::*;
    use crate::controller::Controller;

    #[test]
    fn test_click_match_task() -> Result<(), Box<dyn Error>> {
        let controller = Controller::connect("127.0.0.1:16384")?;
        let click_match_task =
            ClickMatchTask::new(MatchTask::Template("EnterInfrastMistCity.png".to_string()));
        // cost: 1.0240299, min: 1.6783588, max: 7450.5957, loc: (1869, 1146)
        click_match_task.run(&controller)?;
        Ok(())
    }
}

pub enum TaskType {
    BasicTask,
}

pub trait Task {
    type Type;
    fn run(&self, controller: &Controller) -> Result<Self::Type, String>;
}

/// 动作任务
pub enum ActionTask {
    PressEsc,
    PressHome,
    Click(Point<u32>),
    Swipe(Point<u32>, Point<u32>),
}

impl Task for ActionTask {
    type Type = ();
    fn run(&self, controller: &Controller) -> Result<(), String> {
        match self {
            ActionTask::PressEsc => controller.press_esc(),
            ActionTask::PressHome => controller.press_home(),
            ActionTask::Click(p) => controller.click(p.x, p.y),
            ActionTask::Swipe(p1, p2) => {
                controller.swipe((p1.x, p1.y), (p2.x, p2.y), Duration::from_secs(1))
            }
        }
        .map_err(|err| format!("{:?}", err))
    }
}

pub enum MatchTask {
    Template(String), // template_filename
    Ocr(String),      // text
}

impl Task for MatchTask {
    type Type = Rect;
    fn run(&self, controller: &Controller) -> Result<Self::Type, String> {
        let match_type = match self {
            Self::Template(template_filename) => {
                let template = image::open(Path::new("./template").join(template_filename))
                    .map_err(|err| format!("{:?}", err))?
                    .to_luma32f();
                MatchType::TemplateMatch(template)
            }
            Self::Ocr(text) => MatchType::OcrMatch(text.clone()),
        };

        let image = controller.screencap().map_err(|err| format!("{:?}", err))?;
        let image = image.to_luma32f();
        let res = Matcher::new(image, match_type)
            .result()
            .ok_or("match failed".to_string())?;
        Ok(res)
    }
}

// 若任何 Match 失败则失败
// 成功返回所有匹配 Rect
pub struct MultipleMatchTask {
    match_tasks: Vec<MatchTask>,
}

impl MultipleMatchTask {
    pub fn new(match_tasks: Vec<MatchTask>) -> Self {
        Self { match_tasks }
    }
}

impl Task for MultipleMatchTask {
    type Type = Vec<Rect>;
    fn run(&self, controller: &Controller) -> Result<Self::Type, String> {
        self.match_tasks.iter()
            .map(|task| task.run(controller))
            .collect()
    }
}

/// MatchTask 获取 Rect，然后 Controller click_in_rect
pub struct ClickMatchTask {
    match_task: MatchTask,
}

impl ClickMatchTask {
    pub fn new(match_task: MatchTask) -> Self {
        Self { match_task }
    }
}

impl Task for ClickMatchTask {
    type Type = ();
    fn run(&self, controller: &Controller) -> Result<(), String> {
        let rect = self.match_task.run(controller)?;
        controller
            .click_in_rect(rect)
            .map_err(|err| format!("{:?}", err))?;
        Ok(())
    }
}
