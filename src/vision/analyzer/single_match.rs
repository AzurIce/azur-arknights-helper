use std::{ops::RangeInclusive, path::Path};

use aah_cv::template_matching::MatchTemplateMethod;
use image::DynamicImage;

use crate::{
    task::Runnable,
    vision::{
        matcher::single_matcher::{SingleMatcher, SingleMatcherResult},
        utils::{draw_box, Rect},
    }, CachedScreenCapper,
};
use aah_controller::{Controller, DEFAULT_HEIGHT};

use super::matching::MatchOptions;

pub struct SingleMatchAnalyzerOutput {
    pub screen: Box<DynamicImage>,
    pub res: SingleMatcherResult,
    pub annotated_screen: Box<DynamicImage>,
}

/// To find the best result where the template fits in the screen
pub struct SingleMatchAnalyzer {
    options: MatchOptions,
    template: DynamicImage,
}

impl SingleMatchAnalyzer {
    pub fn new(template: DynamicImage) -> Self {
        Self {
            template,
            options: Default::default(),
        }
    }
    pub fn with_color_mask(
        mut self,
        mask_r: RangeInclusive<u8>,
        mask_g: RangeInclusive<u8>,
        mask_b: RangeInclusive<u8>,
    ) -> Self {
        self.options.color_mask = (mask_r, mask_g, mask_b);
        self
    }

    pub fn with_method(mut self, method: MatchTemplateMethod) -> Self {
        self.options.method = Some(method);
        self
    }

    pub fn with_binarize_threshold(mut self, binarize_threshold: u8) -> Self {
        self.options.binarize_threshold = Some(binarize_threshold);
        self
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.options.threshold = Some(threshold);
        self
    }

    pub fn use_cache(mut self) -> Self {
        self.options.use_cache = true;
        self
    }

    pub fn with_roi(mut self, tl: (f32, f32), br: (f32, f32)) -> Self {
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

impl<T: CachedScreenCapper> Runnable<T> for SingleMatchAnalyzer {
    type Res = SingleMatchAnalyzerOutput;
    fn run(&self, runner: &T) -> anyhow::Result<Self::Res> {
        // Get image
        let screen = if self.options.use_cache {
            runner.screen_cache_or_cap()?.clone()
        } else {
            runner
                .screen_cap_and_cache()
                .map_err(|err| anyhow::anyhow!("{:?}", err))?
        };
        self.analyze_image(&screen).map_err(|err| anyhow::anyhow!("{:?}", err))
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use super::*;

    #[test]
    fn test_single_match_analyzer() {
        let root = env::var("CARGO_MANIFEST_DIR").unwrap();
        let root = Path::new(&root);

        let template =
            image::open(root.join("resources/templates/1920x1080/start_start.png")).unwrap();
        let image = image::open(root.join("resources/templates/MUMU-1920x1080/start.png")).unwrap();

        let mut analyzer = SingleMatchAnalyzer::new(template).with_roi((0.3, 0.75), (0.6, 1.0));
        let output = analyzer.analyze_image(&image).unwrap();
        println!("{:?}", output.res.rect);
    }
}
