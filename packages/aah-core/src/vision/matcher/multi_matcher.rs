use std::time::Instant;

use aah_cv::{
    template_matching::{find_matches, match_template, MatchTemplateMethod},
    utils::{luma32f_to_luma8, normalize_luma32f},
};
use color_print::{cformat, cprintln};
use image::{DynamicImage, ImageBuffer, Luma};

use crate::vision::{
    matcher::{
        CCOEFF_THRESHOLD, CCORR_NORMED_THRESHOLD, CCORR_THRESHOLD, SSE_NORMED_THRESHOLD,
        SSE_THRESHOLD,
    },
    utils::Rect,
};

pub enum MultiMatcher {
    Template {
        image: ImageBuffer<Luma<f32>, Vec<f32>>,
        template: ImageBuffer<Luma<f32>, Vec<f32>>,
        method: MatchTemplateMethod,
        threshold: Option<f32>,
    },
}

/// [`MultiMatcher`] 的结果
///
/// - `rects`: 匹配出的矩形框
/// - `matched_img`: 匹配图
pub struct MultiMatcherResult {
    pub rects: Vec<Rect>,
    pub matched_img: Box<DynamicImage>,
}

impl MultiMatcher {
    /// 执行匹配并获取结果
    pub fn result(&self) -> MultiMatcherResult {
        let log_tag = cformat!("<strong>[MultiMatcher]: </strong>");
        match self {
            Self::Template {
                image,
                template,
                method,
                threshold,
            } => {
                let threshold = threshold.unwrap_or(match method {
                    MatchTemplateMethod::SumOfSquaredDifference => SSE_THRESHOLD,
                    MatchTemplateMethod::SumOfSquaredDifferenceNormed => SSE_NORMED_THRESHOLD,
                    MatchTemplateMethod::CrossCorrelation => CCORR_THRESHOLD,
                    MatchTemplateMethod::CrossCorrelationNormed => CCORR_NORMED_THRESHOLD,
                    MatchTemplateMethod::CorrelationCoefficient => CCOEFF_THRESHOLD,
                    MatchTemplateMethod::CorrelationCoefficientNormed => CCOEFF_THRESHOLD,
                });
                // let down_scaled_template = template;
                // let method = MatchTemplateMethod::SumOfSquaredDifference;
                cprintln!(
                    "<dim>{log_tag}image: {}x{}, template: {}x{}, method: {:?}, matching...</dim>",
                    image.width(),
                    image.height(),
                    template.width(),
                    template.height(),
                    method
                );

                // TODO: deal with scale problem, maybe should do it when screen cap stage
                let start_time = Instant::now();
                let mut res = match_template(image, template, *method, false);

                // Normalize
                if matches!(
                    method,
                    MatchTemplateMethod::CorrelationCoefficient
                        | MatchTemplateMethod::CrossCorrelation
                        | MatchTemplateMethod::SumOfSquaredDifference
                ) {
                    res = normalize_luma32f(&res)
                }

                let matched_img = luma32f_to_luma8(&res);
                let matched_img = DynamicImage::ImageLuma8(matched_img);

                let matches = find_matches(&res, template.width(), template.height(), *method, threshold);
                let rects: Vec<Rect> = matches
                    .into_iter()
                    .map(|m| Rect {
                        x: m.location.0,
                        y: m.location.1,
                        width: template.width(),
                        height: template.height(),
                    })
                    .collect();
                cprintln!(
                    "<dim>{log_tag}cost: {}s</dim>",
                    start_time.elapsed().as_secs_f32(),
                );

                return MultiMatcherResult {
                    rects,
                    matched_img: Box::new(matched_img),
                };
            } // TODO: implement OcrMatcher
        }
    }
}

#[cfg(test)]
mod test {
    use aah_cv::template_matching::MatchTemplateMethod;
    use image::math::Rect;

    use crate::vision::{
        matcher::{
            multi_matcher::MultiMatcher,
            test::{get_device_image, get_device_template_prepared, Device},
        },
        utils::{average_hsv_v, draw_box},
    };

    #[test]
    fn test_devices() {
        test_device(Device::MUMU);
    }

    // test to match battle_deploy-card-cost-icon
    fn test_device(device: Device) {
        println!("#### testing device {:?} ####", device);
        for i in 0..=5 {
            test_device_multi_match(Device::MUMU, format!("battle{i}.png"));
        }
    }

    /// deploy available recognize
    fn test_device_multi_match<S: AsRef<str>>(device: Device, image_filename: S) {
        let image_filename = image_filename.as_ref();

        let image = get_device_image(device, image_filename).unwrap();
        let mut res_image = image.clone();
        let template =
            get_device_template_prepared(device, "battle_deploy-card-cost-icon1.png").unwrap();
        let res = MultiMatcher::Template {
            image: image.to_luma32f(),
            template: template.to_luma32f(),
            method: MatchTemplateMethod::CrossCorrelationNormed,
            threshold: Some(0.85),
            // threshold: Some(40.0),
        }
        .result();
        res.matched_img.save(format!("./assets/output/matched_{image_filename}")).unwrap();
        println!("{} matches", res.rects.len());

        let mut cnt = 0;
        for rect in &res.rects {
            let cropped = image.crop_imm(rect.x, rect.y, rect.width, rect.width);
            let avg_hsv_v = average_hsv_v(&cropped);
            // println!("{avg_hsv_v}");
            let color = if avg_hsv_v > 100 {
                [0, 255, 0, 255]
            } else {
                [255, 0, 0, 255]
            };
            draw_box(
                &mut res_image,
                rect.x as i32,
                rect.y as i32,
                rect.width,
                rect.height,
                color,
            );

            let rect = Rect {
                x: rect.x.saturating_add_signed(-80),
                y: rect.y + 10,
                width: 140,
                height: 170,
            };
            // image
            //     .crop_imm(rect.x, rect.y, rect.width, rect.height)
            //     .save(format!("./assets/output/{}.png", cnt))
            //     .unwrap();
            cnt += 1;
            draw_box(
                &mut res_image,
                rect.x as i32,
                rect.y as i32,
                rect.width,
                rect.height,
                [255, 127, 90, 255],
            )
        }
        res_image
            .save(format!("./assets/output/res_{image_filename}"))
            .unwrap();
    }
}
