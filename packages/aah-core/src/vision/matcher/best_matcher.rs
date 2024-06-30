use std::time::Instant;

use aah_cv::{find_extremes, match_template, MatchTemplateMethod};
use color_print::cprintln;
use image::{math::Rect, ImageBuffer, Luma};

use crate::vision::matcher::SSE_THRESHOLD;

/// 匹配器，目前只实现了模板匹配
pub enum BestMatcher {
    Template {
        image: ImageBuffer<Luma<f32>, Vec<f32>>,
        template: ImageBuffer<Luma<f32>, Vec<f32>>,
        threshold: Option<f32>,
    },
    // Ocr {
    //     image: NdTensorBase<f32, Vec<f32>, 3>,
    //     text: String,
    // engine: &'a OcrEngine,
    // },
}

impl BestMatcher {
    /// 执行匹配并获取结果
    pub fn result(&self) -> Option<Rect> {
        match self {
            Self::Template {
                image,
                template,
                threshold,
            } => {
                // let down_scaled_template = template;
                let method = MatchTemplateMethod::SumOfSquaredErrors;
                cprintln!("[Matcher::TemplateMatcher]: image: {}x{}, template: {}x{}, method: {:?}, matching...", image.width(), image.height(), template.width(), template.height(), method);

                // TODO: deal with scale problem, maybe should do it when screen cap stage
                let start_time = Instant::now();
                let res = match_template(image, template, method);
                cprintln!("finding_extremes...");
                let extrems = find_extremes(&res);
                let (x, y) = extrems.min_value_location;
                cprintln!(
                    "[Matcher::TemplateMatcher]: cost: {}s, {:?}",
                    start_time.elapsed().as_secs_f32(),
                    extrems
                );

                if extrems.min_value >= threshold.unwrap_or(SSE_THRESHOLD) {
                    cprintln!("[Matcher::TemplateMatcher]: <red>failed</red>");
                    return None;
                }

                cprintln!("[Matcher::TemplateMatcher]: <green>success!</green>");
                Some(Rect {
                    x,
                    y,
                    width: template.width(),
                    height: template.height(),
                })
            } // TODO: implement OcrMatcher
              // Self::Ocr {
              //     image,
              //     text,
              //     engine,
              // } => {
              //     let ocr = || -> Result<Rect, Box<dyn Error>> {
              //         let ocr_input = engine.prepare_input(image.view())?;

              //         // Phase 1: Detect text words
              //         let word_rects = engine.detect_words(&ocr_input)?;
              //         for rect in &word_rects {
              //             println!("{:?}", rect);
              //         }

              //         // Phase 2: Perform layout analysis
              //         let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

              //         // Phase 3: Recognize text
              //         let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;

              //         for line in line_texts
              //             .iter()
              //             .flatten()
              //             // Filter likely spurious detections. With future model improvements
              //             // this should become unnecessary.
              //             .filter(|l| l.to_string().len() > 1)
              //         {
              //             println!("{}", line);
              //         }
              //         todo!()
              //     };
              //     ocr().ok()
              // }
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use image::math::Rect;

    use crate::vision::matcher::test::Device;

    use super::BestMatcher;


    #[test]
    fn test_device_match() {
        test_device(Device::MUMU);
        // test_device(Device::P40Pro);
    }

    fn test_device(device: Device) {
        println!("#### testing device {:?} ####", device);
        // test_template_matcher_with_device_image(device, "start.png", "start_start.png");

        // test_template_matcher_with_device_image(device, "wakeup.png", "wakeup_wakeup.png");

        // test_template_matcher_with_device_image(device, "main.png", "main_base.png");
        // test_template_matcher_with_device_image(device, "main.png", "main_mission.png");
        // test_template_matcher_with_device_image(device, "main.png", "main_operator.png");
        // test_template_matcher_with_device_image(device, "main.png", "main_squads.png");
        test_template_matcher_with_device_image(device, "main.png", "main_recruit.png");

        // test_template_matcher_with_device_image(device, "notice.png", "close.png");
        // test_template_matcher_with_device_image(device, "mission.png", "back.png");
    }
        fn test_template_matcher_with_device_image(
        device: Device,
        image: &str,
        template: &str,
    ) -> Option<Rect> {
        println!("testing {} on {}...", template, image);
        let templates_path = Path::new("../../resources/templates");
        let image_dir = templates_path.join(device.folder_name());
        let template_dir = templates_path.join("1920x1080");

        let image = image_dir.join(image);
        let template = template_dir.join(template);
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
        BestMatcher::Template {
            image,
            template,
            threshold: None,
        }
        .result()
    }
}