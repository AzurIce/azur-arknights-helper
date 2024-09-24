use std::{sync::Arc, time::Instant};

// use aah_cv::{find_extremes, match_template, MatchTemplateMethod};
use aah_cv::template_matching::{match_template, MatchTemplateMethod};
use color_print::{cformat, cprintln};
use image::{DynamicImage, EncodableLayout, ImageBuffer, Luma};
use imageproc::template_matching::find_extremes;
use ocrs::{ImageSource, OcrEngine};

use crate::vision::{
    matcher::{
        CCOEFF_NORMED_THRESHOLD, CCOEFF_THRESHOLD, CCORR_NORMED_THRESHOLD, CCORR_THRESHOLD,
        SSE_NORMED_THRESHOLD, SSE_THRESHOLD,
    },
    utils::Rect,
};

/// [`SingleMatcher`] 的结果
///
/// - `rect`: 匹配出的矩形框
/// - `matched_img`: 匹配图
pub struct SingleMatcherResult {
    pub rect: Option<Rect>,
    pub matched_img: Box<DynamicImage>,
}

/// 匹配器，目前只实现了模板匹配
pub enum SingleMatcher {
    Template {
        image: ImageBuffer<Luma<f32>, Vec<f32>>,
        template: ImageBuffer<Luma<f32>, Vec<f32>>,
        method: Option<MatchTemplateMethod>,
        threshold: Option<f32>,
    },
    Ocr {
        image: ImageBuffer<Luma<f32>, Vec<f32>>,
        text: String,
        engine: Arc<OcrEngine>,
    },
}

impl SingleMatcher {
    /// 执行匹配并获取结果
    pub fn result(&self) -> SingleMatcherResult {
        // let log_tag = cformat!("[SingleMatcher]: ");
        match self {
            Self::Template {
                image,
                template,
                method,
                threshold,
            } => {
                // let down_scaled_template = template;
                let method = method.unwrap_or(MatchTemplateMethod::SumOfSquaredDifference);
                let threshold = threshold.unwrap_or(match method {
                    MatchTemplateMethod::SumOfSquaredDifference => SSE_THRESHOLD,
                    MatchTemplateMethod::SumOfSquaredDifferenceNormed => SSE_NORMED_THRESHOLD,
                    MatchTemplateMethod::CrossCorrelation => CCORR_THRESHOLD,
                    MatchTemplateMethod::CrossCorrelationNormed => CCORR_NORMED_THRESHOLD,
                    MatchTemplateMethod::CorrelationCoefficient => CCOEFF_THRESHOLD,
                    MatchTemplateMethod::CorrelationCoefficientNormed => CCOEFF_THRESHOLD,
                });
                // cprintln!(
                //     "<dim>{log_tag}image: {}x{}, template: {}x{}, method: {:?}, matching...</dim>",
                //     image.width(),
                //     image.height(),
                //     template.width(),
                //     template.height(),
                //     method
                // );

                // TODO: deal with scale problem, maybe should do it when screen cap stage
                // let start_time = Instant::now();
                let res = match_template(image, template, method, false);

                // Normalize
                let min = res
                    .as_raw()
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let max = res
                    .as_raw()
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let data = res
                    .as_raw()
                    .iter()
                    .map(|x| (x - min) / (max - min))
                    .collect::<Vec<f32>>();

                let matched_img = ImageBuffer::from_vec(
                    res.width(),
                    res.height(),
                    data.iter().map(|p| (p * 255.0) as u8).collect::<Vec<u8>>(),
                )
                .unwrap();
                let matched_img = DynamicImage::ImageLuma8(matched_img);

                let extrems = find_extremes(&res);
                // cprintln!(
                //     "<dim>{log_tag}cost: {}s, {:?}</dim>",
                //     start_time.elapsed().as_secs_f32(),
                //     extrems
                // );

                let success = match method {
                    MatchTemplateMethod::SumOfSquaredDifference
                    | MatchTemplateMethod::SumOfSquaredDifferenceNormed => {
                        extrems.min_value <= threshold
                    }
                    MatchTemplateMethod::CrossCorrelation
                    | MatchTemplateMethod::CrossCorrelationNormed
                    | MatchTemplateMethod::CorrelationCoefficient
                    | MatchTemplateMethod::CorrelationCoefficientNormed => {
                        extrems.max_value >= threshold
                    }
                };

                let rect = if !success {
                    // cprintln!("{log_tag}<red>failed</red>");
                    None
                } else {
                    // cprintln!("{log_tag}<green>success!</green>");
                    let (x, y) = match method {
                        MatchTemplateMethod::SumOfSquaredDifference
                        | MatchTemplateMethod::SumOfSquaredDifferenceNormed => {
                            extrems.min_value_location
                        }
                        MatchTemplateMethod::CrossCorrelation
                        | MatchTemplateMethod::CrossCorrelationNormed
                        | MatchTemplateMethod::CorrelationCoefficient
                        | MatchTemplateMethod::CorrelationCoefficientNormed => {
                            extrems.max_value_location
                        }
                    };
                    Some(Rect {
                        x,
                        y,
                        width: template.width(),
                        height: template.height(),
                    })
                };

                SingleMatcherResult {
                    rect,
                    matched_img: Box::new(matched_img),
                }
            }
            SingleMatcher::Ocr {
                image,
                text,
                engine,
            } => {
                let image_source =
                    ImageSource::from_bytes(image.as_bytes(), image.dimensions()).unwrap();
                let ocr_input = engine.prepare_input(image_source).unwrap();

                let word_rects = engine.detect_words(&ocr_input).unwrap();
                let text_lines = engine.find_text_lines(&ocr_input, &word_rects);
                let text = engine.recognize_text(&ocr_input, &text_lines).unwrap();
                for (text, rect) in text
                    .iter()
                    .zip(text_lines.iter())
                    .filter_map(|(text, rect)| match text {
                        Some(text) => Some((text, rect)),
                        None => None,
                    })
                {
                    println!("{} {:?}", text, rect)
                }

                let rect = Some(Rect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                });
                SingleMatcherResult {
                    rect,
                    matched_img: Box::new(image.clone().into()),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use std::sync::Arc;

    use ocrs::{OcrEngine, OcrEngineParams};
    use rten::Model;

    use crate::vision::matcher::test::{get_device_image, get_device_template_prepared, Device};

    use super::SingleMatcher;

    #[test]
    fn test_ocr() {
        let engine = OcrEngine::new(OcrEngineParams {
            detection_model: Some(
                Model::load_file("../../resources/models/text-detection.rten").unwrap(),
            ),
            recognition_model: Some(
                Model::load_file("../../resources/models/text-recognition.rten").unwrap(),
            ),
            ..Default::default()
        })
        .unwrap();
        let engine = Arc::new(engine);

        let image = get_device_image(Device::MUMU, "episode-13-levels.png").unwrap();
        let image = image.crop_imm(1423, 982, 98, 123);
        image.save("./test.png").unwrap();
        let matcher = SingleMatcher::Ocr {
            image: image.to_luma32f(),
            text: "text".to_string(),
            engine,
        };
        let res = matcher.result();
        // println!("{:?}", res);
    }

    #[test]
    fn test_devices() {
        test_device_match(Device::MUMU);
        // test_device_match(Device::P40Pro);
    }

    fn test_device_match(device: Device) {
        println!("#### testing device {:?} ####", device);
        test_device_single_match(device, "start.png", "start_start.png");

        test_device_single_match(device, "wakeup.png", "wakeup_wakeup.png");

        test_device_single_match(device, "main.png", "main_base.png");
        test_device_single_match(device, "main.png", "main_mission.png");
        test_device_single_match(device, "main.png", "main_operator.png");
        test_device_single_match(device, "main.png", "main_squads.png");
        test_device_single_match(device, "main.png", "main_recruit.png");

        test_device_single_match(device, "notice.png", "notice_close.png");
        test_device_single_match(device, "mission.png", "back.png");

        // fail
        test_device_single_match(device, "start.png", "main_base.png");
        test_device_single_match(device, "start.png", "main_mission.png");
        test_device_single_match(device, "start.png", "main_operator.png");
        test_device_single_match(device, "start.png", "main_squads.png");
        test_device_single_match(device, "start.png", "main_recruit.png");
    }

    fn test_device_single_match<S: AsRef<str>>(
        device: Device,
        image_filename: S,
        template_filename: S,
    ) {
        let image_filename = image_filename.as_ref();
        let template_filename = template_filename.as_ref();
        println!(
            "== testing {} with {} ==",
            template_filename, image_filename
        );

        let image = get_device_image(device, image_filename).unwrap();
        let template = get_device_template_prepared(device, template_filename).unwrap();
        let res = SingleMatcher::Template {
            image: image.to_luma32f(),
            template: template.to_luma32f(),
            threshold: None,
            method: None,
        }
        .result();
        println!("{:?}", res.rect);
    }
}
