use image::{math::Rect, DynamicImage};

use crate::{controller::DEFAULT_HEIGHT, vision::{matcher::multi_matcher::MultiMatcher, utils::binarize_image}, AAH};

use super::Analyzer;

#[derive(Debug)]
pub struct MultiMatchAnalyzerOutput {
    pub screen: DynamicImage,
    pub rects: Vec<Rect>,
}

pub struct MultiMatchAnalyzer {
    template_filename: String,
    binarize_threshold: Option<u8>,
    threshold: Option<f32>,
}

impl MultiMatchAnalyzer {
    pub fn new(
        template_filename: String,
        binarize_threshold: Option<u8>,
        threshold: Option<f32>,
    ) -> Self {
        Self {
            template_filename,
            binarize_threshold,
            threshold,
        }
    }
}

impl Analyzer for MultiMatchAnalyzer {
    type Output = MultiMatchAnalyzerOutput;
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
        let screen = core
            .controller
            .screencap()
            .map_err(|err| format!("{:?}", err))?;

        let template = core.get_template(&self.template_filename).unwrap();

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

        let mut image = screen.clone();
        let mut template = template;
        if let Some(threshold) = self.binarize_threshold {
            image = binarize_image(&image, threshold);
            template = binarize_image(&template, threshold);
        }

        let rects = MultiMatcher::Template {
            image: image.to_luma32f(),
            template: template.to_luma32f(),
            threshold: self.threshold,
        }
        .result()
        .ok_or("match failed".to_string())?;
        Ok(Self::Output { screen, rects })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        vision::analyzer::{multi_match::MultiMatchAnalyzer, Analyzer},
        AAH,
    };

    #[test]
    fn test_multi_template_match_analyzer() {
        let mut core = AAH::connect("127.0.0.1:16384", "../../resources").unwrap();
        let mut analyzer = MultiMatchAnalyzer::new(
            "battle_deploy-card-cost0".to_string(),
            Some(127),
            None,
        );
        let output = analyzer.analyze(&mut core).unwrap();
        println!("{:?}", output);
    }
}
