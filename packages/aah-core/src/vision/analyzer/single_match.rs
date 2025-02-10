use std::{ops::RangeInclusive, path::Path};

use aah_cv::template_matching::MatchTemplateMethod;
use image::DynamicImage;

use crate::{
    utils::resource::get_template,
    vision::{
        matcher::single_matcher::{SingleMatcher, SingleMatcherResult},
        utils::{draw_box, Rect},
    },
    AAH,
};
use aah_controller::DEFAULT_HEIGHT;

use super::{matching::MatchOptions, Analyzer};

pub struct SingleMatchAnalyzerOutput {
    pub screen: Box<DynamicImage>,
    pub res: SingleMatcherResult,
    pub annotated_screen: Box<DynamicImage>,
}

/// To find the best result where the template fits in the screen
pub struct SingleMatchAnalyzer {
    /// filename in `resources/templates`
    template_filename: String,
    options: MatchOptions,
    /// this is loaded from `template_filename`
    template: DynamicImage,
}

impl SingleMatchAnalyzer {
    pub fn new<S: AsRef<str>, P: AsRef<Path>>(res_dir: P, template_filename: S) -> Self {
        let template = get_template(&template_filename, &res_dir).unwrap();
        Self {
            template,
            template_filename: template_filename.as_ref().to_string(),
            options: Default::default(),
        }
    }
    pub fn color_mask(
        mut self,
        mask_r: RangeInclusive<u8>,
        mask_g: RangeInclusive<u8>,
        mask_b: RangeInclusive<u8>,
    ) -> Self {
        self.options.color_mask = (mask_r, mask_g, mask_b);
        self
    }

    pub fn method(mut self, method: MatchTemplateMethod) -> Self {
        self.options.method = Some(method);
        self
    }

    pub fn binarize_threshold(mut self, binarize_threshold: u8) -> Self {
        self.options.binarize_threshold = Some(binarize_threshold);
        self
    }

    pub fn threshold(mut self, threshold: f32) -> Self {
        self.options.threshold = Some(threshold);
        self
    }

    pub fn use_cache(mut self) -> Self {
        self.options.use_cache = true;
        self
    }

    pub fn roi(mut self, tl: (f32, f32), br: (f32, f32)) -> Self {
        self.options.roi = [tl, br];
        self
    }

    pub fn analyze_image(&self, image: &DynamicImage) -> Result<SingleMatchAnalyzerOutput, String> {
        // Scaling
        let template = if image.height() != DEFAULT_HEIGHT {
            let scale_factor = image.height() as f32 / DEFAULT_HEIGHT as f32;

            let new_width = (self.template.width() as f32 * scale_factor) as u32;
            let new_height = (self.template.height() as f32 * scale_factor) as u32;

            DynamicImage::ImageRgba8(image::imageops::resize(
                &self.template,
                new_width,
                new_height,
                image::imageops::FilterType::Lanczos3,
            ))
        } else {
            self.template.clone()
        };

        // Preprocess and match
        let res = {
            let (image, template) = self.options.preprocess(image, &template);
            SingleMatcher::Template {
                image: image.to_luma32f(), // use cropped
                template: template.to_luma32f(),
                method: self.options.method,
                threshold: self.options.threshold,
            }
            .result()
        };

        let [tl, _] = self.options.calc_roi(image);
        let res = SingleMatcherResult {
            rect: res.rect.map(|rect| Rect {
                x: rect.x + tl.0,
                y: rect.y + tl.1,
                ..rect
            }),
            ..res
        };

        // Annotated
        let mut annotated_screen = image.clone();
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

        // println!("cost: {:?}", t.elapsed());
        let screen = Box::new(image.clone());
        let annotated_screen = Box::new(annotated_screen);
        Ok(SingleMatchAnalyzerOutput {
            screen,
            res,
            annotated_screen,
        })
    }
}

impl Analyzer for SingleMatchAnalyzer {
    type Output = SingleMatchAnalyzerOutput;
    fn analyze(&mut self, core: &AAH) -> Result<Self::Output, String> {
        // Get image
        let screen = if self.options.use_cache {
            core.screen_cache_or_cap()?.clone()
        } else {
            core.screen_cap_and_cache()
                .map_err(|err| format!("{:?}", err))?
        };
        self.analyze_image(&screen)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::aah_for_test;

    #[test]
    fn test_single_match_analyzer() {
        let aah = aah_for_test();
        let mut analyzer = SingleMatchAnalyzer::new(&aah.resource.root, "start_start.png")
            .roi((0.3, 0.75), (0.6, 1.0));
        let output = analyzer.analyze(&aah).unwrap();
        println!("{:?}", output.res.rect);
    }
}
