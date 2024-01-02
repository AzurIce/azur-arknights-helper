use std::time::Instant;

use image::{imageops::crop_imm, math::Rect, GrayImage, ImageBuffer, Luma};
// use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};
use template_matching::{find_extremes, match_template, Image, MatchTemplateMethod};

#[cfg(test)]
mod test {
    use image::imageops::crop_imm;

    use super::Matcher;

    #[test]
    fn test_template_matcher() {
        let image = image::open("./output.png")
            .expect("failed to read image")
            .to_luma32f();

        let image = crop_imm(&image, 1833, 1071, 484, 359).to_image();

        let template = image::open("./template/EnterInfrastMistCity.png")
            .expect("failed to read template")
            .to_luma32f();

        let res = Matcher::TemplateMatcher { image, template }.result();
        println!("{:?}", res)
    }
}

pub enum MatchType {
    TemplateMatch(ImageBuffer<Luma<f32>, Vec<f32>>), // template_filename
    OcrMatch(String),                                // target_text
}

/// 匹配器，目前只实现了模板匹配
pub enum Matcher {
    TemplateMatcher {
        image: ImageBuffer<Luma<f32>, Vec<f32>>,
        template: ImageBuffer<Luma<f32>, Vec<f32>>,
    },
    OcrMatcher {
        text: String,
    },
}

const THRESHOLD: f32 = 100.0;

impl Matcher {
    pub fn new(image: ImageBuffer<Luma<f32>, Vec<f32>>, match_type: MatchType) -> Self {
        match match_type {
            MatchType::TemplateMatch(template) => Self::TemplateMatcher { image, template },
            MatchType::OcrMatch(text) => Self::OcrMatcher { text },
        }
    }

    /// 执行匹配并获取结果
    pub fn result(&self) -> Option<Rect> {
        match self {
            Self::TemplateMatcher { image, template } => {
                let method = MatchTemplateMethod::SumOfSquaredDifferences;
                println!("[Matcher::TemplateMatcher]: image: {}x{}, template: {}x{}, template: {:?}, matching...", image.width(), image.height(), template.width(), template.height(), method);

                // TODO: deal with scale problem, maybe should do it when screen cap stage
                let start_time = Instant::now();
                let res = match_template(image, template, method);
                let extrems = find_extremes(&res);
                let (x, y) = extrems.min_value_location;
                println!(
                    "[Matcher::TemplateMatcher]: done! cost: {}s, min: {:?}, max: {:?}, loc: {:?}",
                    start_time.elapsed().as_secs_f32(),
                    extrems.min_value,
                    extrems.max_value,
                    extrems.min_value_location
                );

                if extrems.min_value >= THRESHOLD {
                    return None;
                }

                Some(Rect {
                    x,
                    y,
                    width: template.width(),
                    height: template.height(),
                })
            }
            // TODO: implement OcrMatcher
            OcrMatcher(text) => {
                None
            }
        }
    }
}
