use std::{cmp::Ordering, time::Instant};

use aah_cv::{ccoeff_normed, find_extremes, match_template, MatchTemplateMethod};
use color_print::{cformat, cprintln};
use image::{ImageBuffer, Luma};

use crate::vision::{
    matcher::{CCOEFF_THRESHOLD, SSE_THRESHOLD, THRESHOLD},
    utils::Rect,
};

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
        let log_tag = cformat!("[BestMatcher::TemplateMatcher]: ");
        match self {
            Self::Template {
                image,
                template,
                threshold,
            } => {
                // let down_scaled_template = template;
                let method = MatchTemplateMethod::SumOfSquaredErrors;
                cprintln!("<dim>{log_tag}image: {}x{}, template: {}x{}, method: {:?}, matching...</dim>", image.width(), image.height(), template.width(), template.height(), method);

                // TODO: deal with scale problem, maybe should do it when screen cap stage
                let start_time = Instant::now();
                let res = match_template(image, template, method);

                let data = res.data.to_vec();
                let min = data
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let max = data
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let res_image: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_vec(
                    res.width,
                    res.height,
                    res.data
                        .to_vec()
                        .into_iter()
                        .map(|x| ((x - min) / (max - min) * 255.0) as u8)
                        .collect(),
                )
                .unwrap();
                res_image.save("./match_result.png").unwrap();
                // let res = ccoeff_normed(image, template);
                // let res = imageproc::template_matching::match_template_parallel(
                //     &image_u8,
                //     &template_u8,
                //     imageproc::template_matching::MatchTemplateMethod::CrossCorrelationNormalized,
                // );
                // cprintln!("finding_extremes...");
                // let extrems = find_extremes(&(&res).into());
                let extrems = find_extremes(&res);
                cprintln!(
                    "<dim>{log_tag}cost: {}s, {:?}</dim>",
                    start_time.elapsed().as_secs_f32(),
                    extrems
                );

                match method {
                    MatchTemplateMethod::SumOfSquaredErrors => {
                        if extrems.min_value >= threshold.unwrap_or(SSE_THRESHOLD) {
                            cprintln!("{log_tag}<red>failed</red>");
                            return None;
                        }
                    }
                    MatchTemplateMethod::CrossCorrelation => {
                        if extrems.max_value <= threshold.unwrap_or(THRESHOLD) {
                            cprintln!("{log_tag}<red>failed</red>");
                            return None;
                        }
                    }
                    MatchTemplateMethod::CCOEFF => {
                        if extrems.max_value <= threshold.unwrap_or(CCOEFF_THRESHOLD) {
                            cprintln!("{log_tag}<red>failed</red>");
                            return None;
                        }
                    }
                    MatchTemplateMethod::CCOEFF_NORMED => {
                        if extrems.max_value <= threshold.unwrap_or(THRESHOLD) {
                            cprintln!("{log_tag}<red>failed</red>");
                            return None;
                        }
                    }
                    _ => (),
                };

                cprintln!("{log_tag}<green>success!</green>");
                let (x, y) = match method {
                    MatchTemplateMethod::SumOfSquaredErrors => extrems.min_value_location,
                    MatchTemplateMethod::CrossCorrelation => extrems.max_value_location,
                    MatchTemplateMethod::CCOEFF => extrems.max_value_location,
                    MatchTemplateMethod::CCOEFF_NORMED => extrems.max_value_location,
                    _ => panic!("not implemented"),
                };
                Some(Rect {
                    x,
                    y,
                    width: template.width(),
                    height: template.height(),
                })
            }
        }
    }
}

#[cfg(test)]
mod test {

    use crate::vision::matcher::test::{get_device_image, get_device_template_prepared, Device};

    use super::BestMatcher;

    #[test]
    fn test_devices() {
        test_device_match(Device::MUMU);
        // test_device(Device::P40Pro);
    }

    fn test_device_match(device: Device) {
        println!("#### testing device {:?} ####", device);
        test_device_best_match(device, "start.png", "start_start.png");

        test_device_best_match(device, "wakeup.png", "wakeup_wakeup.png");

        test_device_best_match(device, "main.png", "main_base.png");
        test_device_best_match(device, "main.png", "main_mission.png");
        test_device_best_match(device, "main.png", "main_operator.png");
        test_device_best_match(device, "main.png", "main_squads.png");
        test_device_best_match(device, "main.png", "main_recruit.png");

        test_device_best_match(device, "notice.png", "close.png");
        test_device_best_match(device, "mission.png", "back.png");

        // fail
        test_device_best_match(device, "start.png", "main_base.png");
        test_device_best_match(device, "start.png", "main_mission.png");
        test_device_best_match(device, "start.png", "main_operator.png");
        test_device_best_match(device, "start.png", "main_squads.png");
        test_device_best_match(device, "start.png", "main_recruit.png");
    }

    fn test_device_best_match<S: AsRef<str>>(
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
        let res = BestMatcher::Template {
            image: image.to_luma32f(),
            template: template.to_luma32f(),
            threshold: None,
        }
        .result();
        println!("{:?}", res);
    }
}
