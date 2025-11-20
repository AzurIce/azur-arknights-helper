use std::path::Path;

use ap_controller::DEFAULT_HEIGHT;
use ap_cv::matcher::{MultiMatcher, MultiMatcherResult};
use auto_play::actions::Runnable;
use image::{math::Rect, DynamicImage};

use crate::{utils::resource::get_template, vision::utils::draw_box, AahCore, CachedScreenCapper};

use super::matching::MatchOptions;

pub struct MultiMatchAnalyzerOutput {
    pub screen: Box<DynamicImage>,
    pub res: MultiMatcherResult,
    pub template_size: (u32, u32),
    pub annotated_screen: Box<DynamicImage>,
}

pub struct MultiMatchAnalyzer {
    template: DynamicImage,
    options: MatchOptions,
}

impl MultiMatchAnalyzer {
    pub fn new(res_dir: impl AsRef<Path>, template_path: impl AsRef<Path>) -> Self {
        let template = get_template(template_path, res_dir).unwrap();
        Self {
            template,
            options: Default::default(),
        }
    }

    pub fn with_options(mut self, options: MatchOptions) -> Self {
        self.options = options;
        self
    }

    pub fn analyze_image(&self, image: &DynamicImage) -> anyhow::Result<MultiMatchAnalyzerOutput> {
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
        let [tl, _] = self.options.calc_roi(image);
        let mut res = {
            let (image, template) = self.options.preprocess(image, &template);
            let image = image.to_luma32f();
            let template = template.to_luma32f();
            let options = self
                .options
                .method
                .map(|m| {
                    let options = ap_cv::matcher::MatcherOptions::method_default(m.into());
                    if let Some(threshold) = self.options.threshold {
                        options.with_threshold(threshold)
                    } else {
                        options
                    }
                })
                .unwrap_or_default();
            MultiMatcher::match_template(&image, &template, &options)
        };
        res.result.iter_mut().for_each(|m| {
            m.rect = Rect {
                x: m.rect.x + tl.0,
                y: m.rect.y + tl.1,
                ..m.rect
            };
        });

        // Annotate
        let mut annotated_screen = image.clone();
        for m in &res.result {
            draw_box(
                &mut annotated_screen,
                m.rect.x as i32,
                m.rect.y as i32,
                template.width(),
                template.height(),
                [255, 0, 0, 255],
            );
        }

        // cprintln!("{log_tag}cost: {:?}", t.elapsed());
        let screen = Box::new(image.clone());
        let annotated_screen = Box::new(annotated_screen);
        Ok(MultiMatchAnalyzerOutput {
            screen,
            res,
            template_size: (template.width(), template.height()),
            annotated_screen,
        })
    }
}

impl Runnable<AahCore> for MultiMatchAnalyzer {
    type Output = MultiMatchAnalyzerOutput;
    fn execute(&self, executor: &AahCore) -> anyhow::Result<Self::Output> {
        let screen = if self.options.use_cache {
            executor.screen_cache_or_cap()?.clone()
        } else {
            executor
                .screen_cap_and_cache()
                .map_err(|err| anyhow::anyhow!("{:?}", err))?
        };
        self.analyze_image(&screen)
            .map_err(|err| anyhow::anyhow!(err))
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::vision::analyzer::multi_match::MultiMatchAnalyzer;

    #[test]
    fn test_multi_template_match_analyzer() {
        let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let root = Path::new(&root);

        // let mut core = AAH::connect("127.0.0.1:16384", "../../resources", |_| {}).unwrap();
        let image =
            image::open(root.join("aah-resources/templates/MUMU-1920x1080/1-4.png")).unwrap();
        let mut analyzer =
            MultiMatchAnalyzer::new(root.join("aah-resources"), "battle_deploy-card-cost1.png");
        let output = analyzer.analyze_image(&image).unwrap();
        output.annotated_screen.save("./assets/output.png").unwrap();
        println!("{:?}", output.res.result);
    }
}
