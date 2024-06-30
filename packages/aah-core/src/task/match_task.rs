use image::math::Rect;
use serde::{Deserialize, Serialize};

use crate::{
    controller::DEFAULT_HEIGHT,
    vision::analyzer::{template_match::TemplateMatchAnalyzer, Analyzer},
    AAH,
};

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

        let res = match self {
            Self::Template(template_filename) => {
                let mut analyzer = TemplateMatchAnalyzer::new(template_filename.to_string());
                analyzer.analyze(aah).unwrap().rect
            }
            Self::Ocr(text) => {
                return Err("not implemented".to_string());
                // let image = convert_image_to_ten(image)
                //     .map_err(|err| format!("failed to convert image to tensor: {:?}", err))?;
                // if let Some(ocr_engine) = &aah.ocr_engine {
                //     Matcher::Ocr {
                //         image,
                //         text: text.clone(),
                //         engine: ocr_engine,
                //     }
                // } else {
                //     return Err("".to_string());
                // }
            }
        };

        Ok(res)
    }
}
