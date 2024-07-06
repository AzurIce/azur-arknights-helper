use image::DynamicImage;

use crate::{
    controller::DEFAULT_HEIGHT,
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

pub struct SingleMatchAnalyzer {
    template_filename: String,
    use_cache: bool,
    roi: [(f32, f32); 2], // topleft and bottomright
}

impl SingleMatchAnalyzer {
    pub fn new(template_filename: String) -> Self {
        Self {
            template_filename,
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
        let screen = if self.use_cache {
            core.screen_cache_or_cap()?.clone()
        } else {
            core.screen_cap_and_cache()
                .map_err(|err| format!("{:?}", err))?
        };
        let tl = (
            self.roi[0].0 * screen.width() as f32,
            self.roi[0].1 * screen.height() as f32,
        );
        let br = (
            self.roi[1].0 * screen.width() as f32,
            self.roi[1].1 * screen.height() as f32,
        );
        let tl = (tl.0 as u32, tl.1 as u32);
        let br = (br.0 as u32, br.1 as u32);

        let cropped = screen.crop_imm(tl.0, tl.1, br.0 - tl.0, br.1 - tl.1);
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
mod test {
    use crate::AAH;
    use super::*;

    #[test]
    fn test_single_match_analyzer() {
        let aah = AAH::connect("127.0.0.1:16384", "../../resources", |_|{}).unwrap();
        let mut analyzer = SingleMatchAnalyzer::new("start_start.png".to_string()).roi((0.3, 0.75), (0.6, 1.0));
        let output = analyzer.analyze(&aah).unwrap();
        println!("{:?}", output.res.rect);
    }   
}
