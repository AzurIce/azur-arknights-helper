use std::ops::RangeInclusive;

use aah_cv::template_matching::MatchTemplateMethod;
use image::DynamicImage;

use crate::{
    task::Runnable,
    vision::{
        matcher::multi_matcher::{MultiMatcher, MultiMatcherResult},
        utils::{draw_box, Rect},
    },
    CachedScreenCapper,
};
use aah_controller::DEFAULT_HEIGHT;

use super::matching::MatchOptions;

pub struct MultiMatchAnalyzerOutput {
    pub screen: Box<DynamicImage>,
    pub res: MultiMatcherResult,
    pub annotated_screen: Box<DynamicImage>,
}

pub struct MultiMatchAnalyzer {
    options: MatchOptions,
    template: DynamicImage,
}

impl MultiMatchAnalyzer {
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

    pub fn analyze_image(&self, image: &DynamicImage) -> Result<MultiMatchAnalyzerOutput, String> {
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
            MultiMatcher::Template {
                image: image.to_luma32f(), // use cropped
                template: template.to_luma32f(),
                method: MatchTemplateMethod::CrossCorrelationNormed,
                threshold: self.options.threshold,
            }
            .result()
        };

        let [tl, _] = self.options.calc_roi(image);
        let res = MultiMatcherResult {
            rects: res
                .rects
                .into_iter()
                .map(|rect| Rect {
                    x: rect.x + tl.0,
                    y: rect.y + tl.1,
                    ..rect
                })
                .collect(),
            ..res
        };

        // Annotate
        let mut annotated_screen = image.clone();
        for rect in &res.rects {
            draw_box(
                &mut annotated_screen,
                rect.x as i32,
                rect.y as i32,
                rect.width,
                rect.height,
                [255, 0, 0, 255],
            );
        }

        // cprintln!("{log_tag}cost: {:?}", t.elapsed());
        let screen = Box::new(image.clone());
        let annotated_screen = Box::new(annotated_screen);
        Ok(MultiMatchAnalyzerOutput {
            screen,
            res,
            annotated_screen,
        })
    }
}

impl<T: CachedScreenCapper> Runnable<T> for MultiMatchAnalyzer {
    type Res = MultiMatchAnalyzerOutput;
    fn run(&self, runner: &T) -> anyhow::Result<Self::Res> {
        let screen = if self.options.use_cache {
            runner.screen_cache_or_cap()?.clone()
        } else {
            runner
                .screen_cap_and_cache()
                .map_err(|err| anyhow::anyhow!("{:?}", err))?
        };
        self.analyze_image(&screen)
            .map_err(|err| anyhow::anyhow!(err))
    }
}

#[cfg(test)]
mod test {
    use crate::vision::analyzer::multi_match::MultiMatchAnalyzer;

    #[test]
    fn test_multi_template_match_analyzer() {
        // let mut core = AAH::connect("127.0.0.1:16384", "../../resources", |_| {}).unwrap();
        let template =
            image::open("../../resources/templates/1920x1080/battle_deploy-card-cost1.png")
                .unwrap();
        let image = image::open("../../resources/templates/MUMU-1920x1080/1-4.png").unwrap();
        let mut analyzer = MultiMatchAnalyzer::new(template);
        let output = analyzer.analyze_image(&image).unwrap();
        output.annotated_screen.save("./assets/output.png").unwrap();
        println!("{:?}", output.res.rects);
    }
}
