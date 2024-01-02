use std::path::Path;

use crate::{controller::Controller, vision::matcher::Matcher};

#[cfg(test)]
mod test {
    use std::error::Error;

    use crate::controller::Controller;

    use super::ClickMatchTask;

    #[test]
    fn test_click_match_task() -> Result<(), Box<dyn Error>>{
        let controller = Controller::connect("127.0.0.1:16384")?;
        let click_match_task = ClickMatchTask::new("EnterInfrastMistCity.png");
        click_match_task.run(&controller)?;
        Ok(())
    }
}

pub enum TaskType {
    BasicTask,
}

pub trait Task {}

pub struct ClickMatchTask {
    template_filename: String,
}

impl ClickMatchTask {
    pub fn new<S: AsRef<str>>(template_filename: S) -> Self {
        let template_filename = template_filename.as_ref().to_string();
        Self { template_filename }
    }

    pub fn run(&self, controller: &Controller) -> Result<(), String> {
        let image = controller.screencap().map_err(|err| format!("{:?}", err))?;
        let image = image.to_luma32f();
        let template = image::open(Path::new("./template").join(&self.template_filename))
            .map_err(|err| format!("{:?}", err))?
            .to_luma32f();
        let res = Matcher::TemplateMatcher { image, template }.result();
        controller.click_in_rect(res);
        Ok(())
    }
}
