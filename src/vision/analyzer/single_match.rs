use std::path::Path;

use image::DynamicImage;

use crate::{
    task::Runnable,
    utils::{resource::get_template, LazyImage},
    vision::{
        matcher::single_matcher::{SingleMatcher, SingleMatcherResult},
        utils::{draw_box, Rect},
    },
    CachedScreenCapper,
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
    template: DynamicImage,
    // res_dir: PathBuf,
    options: MatchOptions,
}

impl SingleMatchAnalyzer {
    pub fn new(res_dir: impl AsRef<Path>, template_path: impl AsRef<Path>) -> Self {
        let template = get_template(template_path, res_dir).unwrap();
        Self {
            template,
            // res_dir,
            options: Default::default(),
        }
    }

    pub fn with_options(mut self, options: MatchOptions) -> Self {
        self.options = options;
        self
    }

    pub fn analyze_image(&self, image: &DynamicImage) -> anyhow::Result<SingleMatchAnalyzerOutput> {
        // let template = self.template.get_or_load()?;

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

impl<T: CachedScreenCapper> Analyzer<T> for SingleMatchAnalyzer {
    type Res = SingleMatchAnalyzerOutput;
    fn analyze(&mut self, core: &T) -> anyhow::Result<Self::Res> {
        // Get image
        let screen = if self.options.use_cache {
            core.screen_cache_or_cap()?.clone()
        } else {
            core
                .screen_cap_and_cache()
                .map_err(|err| anyhow::anyhow!("{:?}", err))?
        };
        self.analyze_image(&screen)
            .map_err(|err| anyhow::anyhow!("{:?}", err))
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

        let image =
            image::open(root.join("aah-resources/templates/MUMU-1920x1080/start.png")).unwrap();

        let mut analyzer = SingleMatchAnalyzer::new(root.join("aah-resources"), "start_start.png")
            .with_options(MatchOptions::default().with_roi((0.3, 0.75), (0.6, 1.0)));
        let output = analyzer.analyze_image(&image).unwrap();
        println!("{:?}", output.res.rect);
    }
}
