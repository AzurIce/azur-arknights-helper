use std::time::Instant;

use aah_cv::template_matching::{match_template, MatchTemplateMethod};
use color_print::{cformat, cprintln};
use image::DynamicImage;
use imageproc::template_matching::find_extremes;

pub struct BestMatcherResult {}

pub struct BestMatcher {
    images: Vec<DynamicImage>,
    threshold: Option<f32>,
}

impl BestMatcher {
    pub fn new(images: Vec<DynamicImage>, threshold: Option<f32>) -> Self {
        Self { images, threshold }
    }

    pub fn match_with(&self, template: DynamicImage) -> Option<usize> {
        let log_tag = cformat!("[BestMatcher]: ");
        cprintln!(
            "<dim>{log_tag}matching template with {} images</dim>",
            self.images.len()
        );

        let t = Instant::now();
        let (mut max_val, mut max_idx) = (0.0, None);
        for (idx, img) in self.images.iter().enumerate() {
            let res = match_template(
                &img.to_luma32f(),
                &template.to_luma32f(),
                MatchTemplateMethod::CrossCorrelationNormed,
                false,
            );
            let extremes = find_extremes(&res);
            println!("{:?}", extremes.max_value);
            if extremes.max_value > max_val {
                max_val = extremes.max_value;
                max_idx = Some(idx);
                if max_val >= self.threshold.unwrap_or(0.99) {
                    break;
                }
            }
        }
        println!("{:?}", max_idx);
        cprintln!("<dim>{log_tag}cost: {:?}</dim>", t.elapsed());
        max_idx
    }
}
