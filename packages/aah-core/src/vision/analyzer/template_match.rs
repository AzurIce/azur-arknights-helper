use image::math::Rect;

use crate::{controller::DEFAULT_HEIGHT, vision::matcher::Matcher, AAH};

use super::Analyzer;

#[derive(Debug)]
pub struct TemplateMatchAnalyzerOutput {
    pub rect: Rect,
}

pub struct TemplateMatchAnalyzer {
    template_filename: String,
}

impl TemplateMatchAnalyzer {
    pub fn new(template_filename: String) -> Self {
        Self { template_filename }
    }
}

impl Analyzer for TemplateMatchAnalyzer {
    type Output = TemplateMatchAnalyzerOutput;
    fn analyze(&mut self, core: &AAH) -> Result<Self::Output, String> {
        // Make sure that we are in the operation-start page
        println!(
            "[TemplateMatchAnalyzer]: matching {:?}",
            self.template_filename
        );

        // TODO: 并不是一个好主意，缩放大图消耗时间更多，且误差更大
        // TODO: 然而测试了一下，发现缩放模板有时也会导致误差较大 (333.9063)
        // let image = aah
        //     .controller
        //     .screencap_scaled()
        //     .map_err(|err| format!("{:?}", err))?;
        let image = core
            .controller
            .screencap()
            .map_err(|err| format!("{:?}", err))?;

        let image = image.to_luma32f();
        let template = core
            .get_template(&self.template_filename)
            .unwrap()
            .to_luma32f();

        let template = if image.height() != DEFAULT_HEIGHT {
            let scale_factor = image.height() as f32 / DEFAULT_HEIGHT as f32;

            let new_width = (template.width() as f32 * scale_factor) as u32;
            let new_height = (template.height() as f32 * scale_factor) as u32;

            image::imageops::resize(
                &template,
                new_width,
                new_height,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            template
        };

        let res = Matcher::Template { image, template, threshold: None }
            .result()
            .ok_or("match failed".to_string())?;
        Ok(Self::Output { rect: res })
    }
}

#[cfg(test)]
mod test {}
