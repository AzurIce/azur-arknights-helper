use std::{error::Error, time::Instant};

use color_print::cprintln;
use image::{math::Rect, DynamicImage, ImageBuffer, Luma};
use rten_tensor::{NdTensorBase, NdTensorView};
// use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};
use aah_cv::{find_extremes, find_matches, match_template, MatchTemplateMethod};

pub fn convert_image_to_ten(
    image: DynamicImage,
) -> Result<NdTensorBase<f32, Vec<f32>, 3>, Box<dyn Error>> {
    let image = image.into_rgb8();
    let (width, height) = image.dimensions();
    let layout = image.sample_layout();

    let chw_tensor = NdTensorView::from_slice(
        image.as_raw().as_slice(),
        [height as usize, width as usize, 3],
        Some([
            layout.height_stride,
            layout.width_stride,
            layout.channel_stride,
        ]),
    )
    .map_err(|err| format!("failed to convert image to tensorL {:?}", err))?
    .permuted([2, 0, 1]) // HWC => CHW
    .to_tensor() // Make tensor contiguous, which makes `map` faster
    .map(|x| *x as f32 / 255.); // Rescale from [0, 255] to [0, 1]
    Ok(chw_tensor)
}

pub enum MultiMatcher {
    Template {
        image: ImageBuffer<Luma<f32>, Vec<f32>>,
        template: ImageBuffer<Luma<f32>, Vec<f32>>,
        threshold: Option<f32>,
    },
}

impl MultiMatcher {
    /// 执行匹配并获取结果
    pub fn result(&self) -> Option<Vec<Rect>> {
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

                let matches = find_matches(&res, threshold.unwrap_or(SSE_THRESHOLD));
                let matches: Vec<Rect> = matches
                    .into_iter()
                    .map(|m| Rect {
                        x: m.location.0,
                        y: m.location.1,
                        width: template.width(),
                        height: template.height(),
                    })
                    .collect();
                cprintln!(
                    "[Matcher::TemplateMatcher]: cost: {}s,",
                    start_time.elapsed().as_secs_f32(),
                );

                if matches.len() == 0 {
                    cprintln!("[Matcher::TemplateMatcher]: <red>failed</red>");
                    return None;
                }

                cprintln!(
                    "[MultiMatcher::TemplateMatcher]: <green>{} matches</green>",
                    matches.len()
                );
                Some(matches)
            } // TODO: implement OcrMatcher
        }
    }
}

/// 匹配器，目前只实现了模板匹配
pub enum Matcher {
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

const THRESHOLD: f32 = 100.0;
const SSE_THRESHOLD: f32 = 40.0;

impl Matcher {
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
    use std::{error::Error, path::Path};

    use image::{imageops::crop_imm, math::Rect, DynamicImage, ImageBuffer, Luma};

    // use crate::vision::utils::try_init_ocr_engine;

    use crate::vision::utils::{binarize_image, draw_box};

    use super::{convert_image_to_ten, Matcher, MultiMatcher};

    fn get_image<P: AsRef<Path>>(path: P) -> Result<DynamicImage, String> {
        image::open(path).map_err(|err| format!("failed to open image: {:?}", err))
    }

    fn get_device_image<P: AsRef<Path>>(
        device: Device,
        filename: P,
    ) -> Result<DynamicImage, String> {
        let templates_path = Path::new("../../resources/templates");
        let image_path = templates_path.join(device.folder_name());
        get_image(image_path.join(filename))
    }

    fn get_template<P: AsRef<Path>>(filename: P) -> Result<DynamicImage, String> {
        let templates_path = Path::new("../../resources/templates");
        let template_path = templates_path.join("1920x1080");
        get_image(template_path.join(filename))
    }

    fn get_device_template_prepared<P: AsRef<Path>>(
        device: Device,
        filename: P,
    ) -> Result<DynamicImage, String> {
        let orinigal_template = get_template(filename)?;
        let template = orinigal_template;
        let template = {
            let new_width = (template.width() as f32 * device.factor()) as u32;
            let new_height = (template.height() as f32 * device.factor()) as u32;

            DynamicImage::ImageRgba8(image::imageops::resize(
                &template,
                new_width,
                new_height,
                image::imageops::FilterType::Triangle,
            ))
        };
        Ok(template)
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
                Device::MUMU => "MUMU-1920x1080",
                Device::P40Pro => "P40 Pro-2640x1200",
            }
        }
    }

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

    #[test]
    fn test_device_multi_match() {
        test_device_multi(Device::MUMU);
    }

    fn test_device_multi(device: Device) {
        println!("#### testing device {:?} ####", device);

        let image = get_device_image(device, "battle.png").unwrap();
        let mut res_image = image.clone();
        let image = binarize_image(&image, 127);
        image.save("binarized_image.png").unwrap();

        let template =
            get_device_template_prepared(device, "battle_deploy-card-cost-icon0.png").unwrap();
        let template = binarize_image(&template, 127);
        template.save("binarized_template0.png").unwrap();
        let res0 = MultiMatcher::Template {
            image: image.to_luma32f(),
            template: template.to_luma32f(),
            threshold: Some(20.0),
        }
        .result()
        .unwrap();
        println!("0: {} matches", res0.len());
        let template =
            get_device_template_prepared(device, "battle_deploy-card-cost-icon1.png").unwrap();
        let template = binarize_image(&template, 127);
        template.save("binarized_template1.png").unwrap();
        let res1 = MultiMatcher::Template {
            image: image.to_luma32f(),
            template: template.to_luma32f(),
            threshold: Some(10.0),
        }
        .result()
        .unwrap();
        println!("1: {} matches", res1.len());

        for rect in &res0 {
            draw_box(
                &mut res_image,
                rect.x as i32,
                rect.y as i32,
                rect.width,
                rect.height,
                [255, 0, 0, 255],
            )
        }
        for rect in &res1 {
            draw_box(
                &mut res_image,
                rect.x as i32,
                rect.y as i32,
                rect.width,
                rect.height,
                [0, 255, 0, 255],
            )
        }
        res_image.save("./output.png").unwrap();
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
        Matcher::Template {
            image,
            template,
            threshold: None,
        }
        .result()
    }
}
