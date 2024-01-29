use std::path::Path;

use image::math::Rect;
use serde::{Deserialize, Serialize};

use crate::{vision::matcher::{Matcher, convert_image_to_ten}, AAH};

use super::Task;

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

        // TODO: 并不是一个好主意，缩放大图消耗时间更多，且误差更大
        // TODO: 然而测试了一下，发现缩放模板有时也会导致误差较大 (333.9063)
        // let image = aah
        //     .controller
        //     .screencap_scaled()
        //     .map_err(|err| format!("{:?}", err))?;
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
