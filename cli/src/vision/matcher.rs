use image::{imageops::crop_imm, math::Rect, GrayImage, ImageBuffer, Luma};
// use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};
use template_matching::{find_extremes, match_template, MatchTemplateMethod};

#[cfg(test)]
mod test {
    use image::imageops::crop_imm;

    use super::Matcher;

    #[test]
    fn test_template_matcher() {
        let image = image::open("./output.png")
            .expect("failed to read image")
            .to_luma32f();
        println!("target image size: {}, {}", image.width(), image.height());

        let image = crop_imm(&image, 1833, 1071, 484, 359).to_image();
        println!("cropped image size: {}, {}", image.width(), image.height());

        let template =
            image::open("./template/EnterInfrastMistCity.png")
                .expect("failed to read template")
                .to_luma32f();
        println!(
            "template image size: {}, {}",
            template.width(),
            template.height()
        );

        let res = Matcher::TemplateMatcher { image, template }.result();
        println!("{:?}", res)
    }
}

/// 匹配器，目前只实现了模板匹配
pub enum Matcher {
    TemplateMatcher {
        image: ImageBuffer<Luma<f32>, Vec<f32>>,
        template: ImageBuffer<Luma<f32>, Vec<f32>>,
    },
}

impl Matcher {
    /// 执行匹配并获取结果
    pub fn result(&self) -> Rect {
        match self {
            Self::TemplateMatcher { image, template } => {
                println!("matching template...");
                // TODO: deal with scale problem, maybe should do it when screen cap stage
                let res = match_template(image, template, MatchTemplateMethod::SumOfSquaredDifferences);
                println!("finding extrems...");
                let extrems = find_extremes(&res);
                let (x, y) = extrems.min_value_location;

                Rect {
                    x,
                    y,
                    width: template.width(),
                    height: template.height(),
                }
            }
            _ => Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
        }
    }
}
