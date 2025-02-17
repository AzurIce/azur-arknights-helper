use std::path::Path;

use aah_cv::template_matching::MatchTemplateMethod;
use image::DynamicImage;

use crate::{
    task::Runnable,
    utils::{resource::get_template, LazyImage},
    vision::{
        matcher::multi_matcher::{MultiMatcher, MultiMatcherResult},
        utils::{draw_box, Rect},
    },
    CachedScreenCapper,
};
use aah_controller::DEFAULT_HEIGHT;

use super::{matching::MatchOptions, Analyzer};

pub struct MultiMatchAnalyzerOutput {
    pub screen: Box<DynamicImage>,
    pub res: MultiMatcherResult,
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

impl<T: CachedScreenCapper> Analyzer<T> for MultiMatchAnalyzer {
    type Res = MultiMatchAnalyzerOutput;
    fn analyze(&mut self, core: &T) -> anyhow::Result<Self::Res> {
        let screen = if self.options.use_cache {
            core.screen_cache_or_cap()?.clone()
        } else {
            core
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
        println!("{:?}", output.res.rects);
    }
}
