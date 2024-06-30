use std::time::Instant;

use aah_cv::{find_matches, match_template, MatchTemplateMethod};
use color_print::cprintln;
use image::{math::Rect, ImageBuffer, Luma};

use crate::vision::matcher::SSE_THRESHOLD;

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

                let matches = find_matches(
                    &res,
                    template.width(),
                    template.height(),
                    threshold.unwrap_or(SSE_THRESHOLD),
                );
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

#[cfg(test)]
mod test {
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
            threshold: None,
        }
        .result()
        .unwrap();
        println!("{} matches", res.len());

        let mut cnt = 0;
        for rect in &res {
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
                x: rect.x - 80,
                y: rect.y + 10,
                width: 140,
                height: 170,
            };
            image.crop_imm(rect.x, rect.y, rect.width, rect.height).save(format!("./assets/output/{}.png", cnt)).unwrap();
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
