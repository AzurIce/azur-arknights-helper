use image::DynamicImage;

use crate::{
    controller::DEFAULT_HEIGHT,
    vision::{
        matcher::single_matcher::{SingleMatcher, SingleMatcherResult},
        utils::draw_box,
    },
    AAH,
};

use super::Analyzer;

pub struct SingleMatchAnalyzerOutput {
    pub screen: Box<DynamicImage>,
    pub res: SingleMatcherResult,
    pub annotated_screen: Box<DynamicImage>,
}

pub struct SingleMatchAnalyzer {
    template_filename: String,
}

impl SingleMatchAnalyzer {
    pub fn new(template_filename: String) -> Self {
        Self { template_filename }
    }
}

impl Analyzer for SingleMatchAnalyzer {
    type Output = SingleMatchAnalyzerOutput;
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

        // Get image
        let screen = core
            .controller
            .screencap()
            .map_err(|err| format!("{:?}", err))?;
        let template = core.get_template(&self.template_filename).unwrap();

        // Scaling
        let template = if screen.height() != DEFAULT_HEIGHT {
            let scale_factor = screen.height() as f32 / DEFAULT_HEIGHT as f32;

            let new_width = (template.width() as f32 * scale_factor) as u32;
            let new_height = (template.height() as f32 * scale_factor) as u32;

            DynamicImage::ImageRgba8(image::imageops::resize(
                &template,
                new_width,
                new_height,
                image::imageops::FilterType::Lanczos3,
            ))
        } else {
            template
        };

        // Match
        let res = SingleMatcher::Template {
            image: screen.to_luma32f(),
            template: template.to_luma32f(),
            threshold: None,
        }
        .result();

        // Annotated
        let mut annotated_screen = screen.clone();
        if let Some(rect) = &res.rect {
            draw_box(
                &mut annotated_screen,
                rect.x as i32,
                rect.y as i32,
                rect.width,
                rect.height,
                [255, 0, 0, 255],
            );
        }

        let screen = Box::new(screen);
        let annotated_screen = Box::new(annotated_screen);
        Ok(Self::Output {
            screen,
            res,
            annotated_screen,
        })
    }
}

#[cfg(test)]
mod test {}
