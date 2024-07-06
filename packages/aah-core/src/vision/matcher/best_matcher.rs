use std::{sync::Mutex, time::Instant};

use aah_cv::template_matching::cross_correlation_normed::CrossCorrelationNormedMatcher;
use color_print::{cformat, cprintln};
use image::DynamicImage;
use imageproc::template_matching::find_extremes;

pub struct BestMatcherResult {}

pub struct BestMatcher {
    images: Vec<DynamicImage>,
    matcher: Mutex<CrossCorrelationNormedMatcher>,
}

impl BestMatcher {
    pub fn new(images: Vec<DynamicImage>) -> Self {
        Self {
            images,
            matcher: Mutex::new(CrossCorrelationNormedMatcher::new()),
        }
    }

    pub fn match_with(&self, template: DynamicImage) -> usize {
        let log_tag = cformat!("[BestMatcher]: ");
        cprintln!(
            "<dim>{log_tag}matching template with {} images</dim>",
            self.images.len()
        );

        let t = Instant::now();
        let res = self
            .images
            .iter()
            .map(|img| {
                // cprintln!(
                //     "<dim>matching {}x{} with {}x{}...</dim>",
                //     img.width(),
                //     img.height(),
                //     template.width(),
                //     template.height()
                // );
                // let res = ccoeff_normed(&img.to_luma32f(), &template.to_luma32f());
                let res = self
                    .matcher
                    .lock()
                    .unwrap()
                    .match_template(&img.to_luma32f(), &template.to_luma32f());
                let extremes = find_extremes(&res);
                println!("{:?}", extremes);
                extremes.max_value
            })
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        cprintln!("<dim>cost: {:?}</dim>", t.elapsed());
        res.0
    }
}
