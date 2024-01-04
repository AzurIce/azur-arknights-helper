use std::time::Instant;

use image::{math::Rect, ImageBuffer, Luma};
// use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};
use template_matching::{find_extremes, match_template, MatchTemplateMethod};

#[cfg(test)]
mod test {
    use std::path::Path;

    use image::{imageops::crop_imm, math::Rect, ImageBuffer, Luma};

    use super::Matcher;

    #[test]
    fn test_template_matcher() {
        let image = image::open("./output.png")
            .expect("failed to read image")
            .to_luma32f();

        let image = crop_imm(&image, 1833, 1071, 484, 359).to_image();

        let template = image::open("./resource/template/EnterInfrastMistCity.png")
            .expect("failed to read template")
            .to_luma32f();

        let res = Matcher::Template { image, template }.result();
        println!("{:?}", res)
    }

    #[test]
    fn test_template() {
        let res = test_template_matcher_with_image("_2.png", "Confirm.png");
        println!("{:?}", res)
    }

    fn test_template_matcher_with_image(image: &str, template: &str) -> Option<Rect> {
        let path = Path::new("../../resources/templates");
        let image = path.join(image);
        let image = image::open(image)
            .expect("failed to read image")
            .to_luma32f();

        let template = path.join(template);
        let template = image::open(template)
            .expect("failed to read template")
            .to_luma32f();
        Matcher::Template { image, template }.result()
    }
}

/// 匹配器，目前只实现了模板匹配
pub enum Matcher {
    Template {
        image: ImageBuffer<Luma<f32>, Vec<f32>>,
        template: ImageBuffer<Luma<f32>, Vec<f32>>,
    },
    Ocr(String),
}

const THRESHOLD: f32 = 100.0;

impl Matcher {
    /// 执行匹配并获取结果
    pub fn result(&self) -> Option<Rect> {
        match self {
            Self::Template { image, template } => {
                let down_scaled_template = template;
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
            Self::Ocr(text) => None,
        }
    }
}
