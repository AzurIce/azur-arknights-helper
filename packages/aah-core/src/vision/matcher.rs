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
        let res = test_template_matcher_with_image_and_scale_factor("_2.png", "Confirm.png", 1.0);
        println!("{:?}", res)
    }

    #[derive(Debug, Clone, Copy)]
    enum Device {
        MUMU,
        P40Pro,
    }

    impl Device {
        fn factor(&self) -> f32 {
            match self {
                Device::MUMU => 1.0,
                Device::P40Pro => 0.83,
            }
        }
        fn folder_name(&self) -> &str {
            match self {
                Device::MUMU => "MUMU-2560x1440",
                Device::P40Pro => "P40 Pro-2640x1200",
            }
        }
    }

    #[test]
    fn test_device_match() {
        test_device(Device::MUMU);
        test_device(Device::P40Pro);
    }

    fn test_device(device: Device) {
        println!("#### testing device {:?} ####", device);
        test_template_matcher_with_device_image(device, "start.png", "start.png");

        test_template_matcher_with_device_image(device, "wakeup.png", "start_wakeup.png");

        test_template_matcher_with_device_image(device, "main.png", "EnterInfrastMistCity.png");
        test_template_matcher_with_device_image(device, "main.png", "EnterMissionMistCity.png");
        test_template_matcher_with_device_image(device, "main.png", "EnterRecruitMistCity.png");
        test_template_matcher_with_device_image(device, "main.png", "MailBoxIconWhite.png");

        test_template_matcher_with_device_image(device, "mission.png", "CollectAllAward.png");
        test_template_matcher_with_device_image(device, "mission.png", "Close.png");
        test_template_matcher_with_device_image(device, "mission.png", "MissonTagMainTheme.png");
        test_template_matcher_with_device_image(device, "mission.png", "ButtonToggleTopNavigator.png");
        test_template_matcher_with_device_image(device, "mission.png", "award_2.png");
    }

    fn test_template_matcher_with_device_image(
        device: Device,
        image: &str,
        template: &str,
    ) -> Option<Rect> {
        println!("testing {} on {}...", template, image);
        let templates_path = Path::new("../../resources/templates");
        let image_path = templates_path.join(device.folder_name());

        let image = image_path.join(image);
        let template = templates_path.join(template);
        test_template_matcher_with_image_and_scale_factor(image, template, device.factor())
    }

    fn test_template_matcher_with_image_and_scale_factor<P: AsRef<Path>>(
        image: P,
        template: P,
        factor: f32,
    ) -> Option<Rect> {
        let image = image.as_ref();
        let template = template.as_ref();

        let image = image::open(image)
            .expect("failed to read image")
            .to_luma32f();

        let template = image::open(template)
            .expect("failed to read template")
            .to_luma32f();
        let template = {
            let new_width = (template.width() as f32 * factor) as u32;
            let new_height = (template.height() as f32 * factor) as u32;

            image::imageops::resize(
                &template,
                new_width,
                new_height,
                image::imageops::FilterType::Triangle,
            )
        };
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
