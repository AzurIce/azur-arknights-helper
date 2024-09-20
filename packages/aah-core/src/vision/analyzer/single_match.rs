use std::{path::Path, time::Instant};

use color_print::{cformat, cprintln};
use image::DynamicImage;

use crate::{
    controller::DEFAULT_HEIGHT,
    utils::resource::get_template,
    vision::{
        matcher::single_matcher::{SingleMatcher, SingleMatcherResult},
        utils::{draw_box, Rect},
    },
    AAH,
};

use super::Analyzer;

pub struct SingleMatchAnalyzerOutput {
    pub screen: Box<DynamicImage>,
    pub res: SingleMatcherResult,
    pub annotated_screen: Box<DynamicImage>,
}

/// To find the best result where the template fits in the screen
pub struct SingleMatchAnalyzer {
    /// filename in `resources/templates`
    template_filename: String,
    /// Whether should use cache instead of capture a new screen
    use_cache: bool,
    /// Region of intrest (top left and bottom right)
    roi: [(f32, f32); 2],

    /// this is loaded from `template_filename`
    template: DynamicImage,
}

impl SingleMatchAnalyzer {
    pub fn new<S: AsRef<str>, P: AsRef<Path>>(res_dir: P, template_filename: S) -> Self {
        let template = get_template(&template_filename, &res_dir).unwrap();
        Self {
            template,
            template_filename: template_filename.as_ref().to_string(),
            use_cache: false,
            roi: [(0.0, 0.0), (1.0, 1.0)],
        }
    }

    pub fn use_cache(mut self) -> Self {
        self.use_cache = true;
        self
    }

    pub fn roi(mut self, tl: (f32, f32), br: (f32, f32)) -> Self {
        self.roi = [tl, br];
        self
    }

    pub fn analyze_image(&self, image: &DynamicImage) -> Result<SingleMatchAnalyzerOutput, String> {
        // let log_tag = cformat!("<strong>[SingleMatchAnalyzer]: </strong>");
        // cprintln!("{log_tag}matching {:?}", self.template_filename);
        // let t = Instant::now();

        // TODO: 并不是一个好主意，缩放大图消耗时间更多，且误差更大
        // TODO: 然而测试了一下，发现缩放模板有时也会导致误差较大 (333.9063)
        // let image = aah
        //     .controller
        //     .screencap_scaled()
        //     .map_err(|err| format!("{:?}", err))?;
        let tl = (
            self.roi[0].0 * image.width() as f32,
            self.roi[0].1 * image.height() as f32,
        );
        let br = (
            self.roi[1].0 * image.width() as f32,
            self.roi[1].1 * image.height() as f32,
        );
        let tl = (tl.0 as u32, tl.1 as u32);
        let br = (br.0 as u32, br.1 as u32);

        let cropped = image.crop_imm(tl.0, tl.1, br.0 - tl.0, br.1 - tl.1);

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

        // Match
        let res = SingleMatcher::Template {
            image: cropped.to_luma32f(), // ! cropped
            template: template.to_luma32f(),
            threshold: None,
        }
        .result();
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
        let screen = if self.use_cache {
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
    use std::sync::Arc;

    use aah_resource::Resource;

    use super::*;
    use crate::AAH;

    #[test]
    fn test_single_match_analyzer() {
        let resource = Resource::load("../../resources").unwrap();
        let aah = AAH::connect("127.0.0.1:16384", Arc::new(resource)).unwrap();
        let mut analyzer =
            SingleMatchAnalyzer::new(&aah.resource.root, "start_start.png").roi((0.3, 0.75), (0.6, 1.0));
        let output = analyzer.analyze(&aah).unwrap();
        println!("{:?}", output.res.rect);
    }
}
