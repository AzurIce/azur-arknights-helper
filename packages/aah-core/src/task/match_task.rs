use aah_resource::manifest::MatchTask;

use crate::{
    vision::{
        analyzer::{single_match::SingleMatchAnalyzer, Analyzer},
        utils::Rect,
    },
    AAH,
};

use super::{Runnable, TaskEvt};

// TODO: add optional roi field

impl Runnable for MatchTask {
    type Res = Rect;
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, String> {
        println!("[MatchTask]: matching {:?}", self);

        let res = match self {
            Self::Template(template_filename) => {
                let mut analyzer =
                    SingleMatchAnalyzer::new(&aah.resource.root, template_filename.to_string());
                let output = analyzer.analyze(aah)?;

                aah.emit_task_evt(TaskEvt::Log(format!(
                    "[SingleMatchAnalyzer]: {:?}",
                    output.res.rect
                )));
                aah.emit_task_evt(TaskEvt::AnnotatedImg(*output.annotated_screen));

                output.res.rect
            } // Self::Ocr(text) => {
              // return Err("not implemented".to_string());
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
              // }
        }
        .ok_or("match failed".to_string());

        res
    }
}
