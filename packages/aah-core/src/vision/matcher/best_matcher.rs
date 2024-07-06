use std::{sync::Mutex, time::Instant};

use aah_cv::template_matching::match_template_ccorr_normed;
use color_print::{cformat, cprintln};
use image::DynamicImage;
use imageproc::template_matching::find_extremes;

pub struct BestMatcherResult {}

pub struct BestMatcher {
    images: Vec<DynamicImage>,
}

impl BestMatcher {
    pub fn new(images: Vec<DynamicImage>) -> Self {
        Self { images }
    }

    pub fn match_with(&self, template: DynamicImage) -> usize {
        // let log_tag = cformat!("[BestMatcher]: ");
        // cprintln!(
        //     "<dim>{log_tag}matching template with {} images</dim>",
        //     self.images.len()
        // );

        let t = Instant::now();
        let (mut max_val, mut max_idx) = (0.0, 0);
        for (idx, img) in self.images.iter().enumerate() {
            let res = match_template_ccorr_normed(&img.to_luma32f(), &template.to_luma32f());
            let extremes = find_extremes(&res);
            if extremes.max_value > max_val {
                max_val = extremes.max_value;
                max_idx = idx;
                if max_val >= 0.99 {
                    break;
                }
            }
        }
        // cprintln!("<dim>cost: {:?}</dim>", t.elapsed());
        max_idx
    }
}
