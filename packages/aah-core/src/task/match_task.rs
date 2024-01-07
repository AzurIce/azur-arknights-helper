use std::path::Path;

use image::math::Rect;
use serde::{Deserialize, Serialize};

use crate::{controller::Controller, vision::matcher::{Matcher, convert_image_to_ten}, AAH};

use super::{Exec, Task};

/// 动作任务

// 若任何 Match 失败则失败
// 成功返回所有匹配 Rect
#[derive(Debug)]
pub struct AndTask {
    tasks: Vec<Box<dyn Exec>>,
}

impl AndTask {
    pub fn new(tasks: Vec<Box<dyn Exec>>) -> Self {
        Self { tasks }
    }
}

impl Exec for AndTask {
    fn run(&self, controller: &Controller) -> Result<(), String> {
        self.tasks.iter().try_for_each(|task| task.run(controller))
    }
}

// TODO: change the Exec Trait
// 若任何 Match 失败则失败
// 成功返回所有匹配 Rect
#[derive(Debug)]
pub struct OrTask {
    tasks: Vec<Box<dyn Exec>>,
}

impl OrTask {
    pub fn new(tasks: Vec<Box<dyn Exec>>) -> Self {
        Self { tasks }
    }
}

impl Exec for OrTask {
    fn run(&self, controller: &Controller) -> Result<(), String> {
        for task in &self.tasks {
            if task.run(controller).is_ok() {
                return Ok(());
            }
        }
        Err("match failed".to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "template")]
pub enum MatchTask {
    Template(String), // template_filename
    Ocr(String),      // text
}
// TODO: add optional roi field

impl Task for MatchTask {
    type Res = Rect;
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, String> {
        println!("[MatchTask]: matching {:?}", self);
        let image = aah
            .controller
            .screencap()
            .map_err(|err| format!("{:?}", err))?;

        let matcher = match self {
            Self::Template(template_filename) => {
                let image = image.to_luma32f();
                let template =
                    image::open(Path::new("./resources/templates/").join(template_filename))
                        .map_err(|err| format!("{:?}", err))?
                        .to_luma32f();

                let template = if image.height() != 1440 {
                    // let scale_factor = 2560.0 / image.width() as f32;
                    let scale_factor = image.height() as f32 / 1440.0;

                    let new_width = (template.width() as f32 * scale_factor) as u32;
                    let new_height = (template.height() as f32 * scale_factor) as u32;

                    image::imageops::resize(
                        &template,
                        new_width,
                        new_height,
                        image::imageops::FilterType::Triangle,
                    )
                } else {
                    template
                };
                Matcher::Template { image, template }
            }
            Self::Ocr(text) => {
                let image = convert_image_to_ten(image).map_err(|err|format!("failed to convert image to tensor: {:?}", err))?;
                if let Some(ocr_engine) = &aah.ocr_engine {
                    Matcher::Ocr {
                        image,
                        text: text.clone(),
                        engine: ocr_engine,
                    }
                } else {
                    return Err("".to_string());
                }
            }
        };

        let res = matcher.result().ok_or("match failed".to_string())?;
        Ok(res)
    }
}
