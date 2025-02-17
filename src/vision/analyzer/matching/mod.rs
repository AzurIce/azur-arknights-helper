use std::ops::RangeInclusive;

use aah_cv::template_matching::MatchTemplateMethod;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::vision::utils::{binarize_image, mask_image};

/// The generic options for matching
#[derive(Serialize, Deserialize)]
#[skip_serializing_none]
pub struct MatchOptions {
    /// The matching method
    pub(crate) method: Option<MatchTemplateMethod>,
    /// The matching threshold
    pub(crate) threshold: Option<f32>,
    /// Whether should use cached img first
    pub(crate) use_cache: bool,

    /// Color mask
    pub(crate) color_mask: (RangeInclusive<u8>, RangeInclusive<u8>, RangeInclusive<u8>),
    /// Binarization threshold
    pub(crate) binarize_threshold: Option<u8>,
    /// Region of interest represented by top-left and bottom-right pos in [0.0, 1.0]
    pub(crate) roi: [(f32, f32); 2], // topleft and bottomright
}

impl MatchOptions {
    pub fn with_color_mask(
        mut self,
        mask_r: RangeInclusive<u8>,
        mask_g: RangeInclusive<u8>,
        mask_b: RangeInclusive<u8>,
    ) -> Self {
        self.color_mask = (mask_r, mask_g, mask_b);
        self
    }

    pub fn with_method(mut self, method: MatchTemplateMethod) -> Self {
        self.method = Some(method);
        self
    }

    pub fn with_binarize_threshold(mut self, binarize_threshold: u8) -> Self {
        self.binarize_threshold = Some(binarize_threshold);
        self
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = Some(threshold);
        self
    }

    pub fn use_cache(mut self) -> Self {
        self.use_cache = true;
        self
    }

    pub fn with_roi(mut self, tl: (f32, f32), br: (f32, f32)) -> Self {
        self.roi = [tl, br];
        self
    }
}

impl Default for MatchOptions {
    fn default() -> Self {
        Self {
            method: None,
            threshold: None,
            use_cache: false,
            color_mask: (0..=255, 0..=255, 0..=255),
            binarize_threshold: None,
            roi: [(0.0, 0.0), (1.0, 1.0)],
        }
    }
}

impl MatchOptions {
    pub fn calc_roi(&self, image: &DynamicImage) -> [(u32, u32); 2] {
        let tl = (
            self.roi[0].0 * image.width() as f32,
            self.roi[0].1 * image.height() as f32,
        );
        let br = (
            self.roi[1].0 * image.width() as f32,
            self.roi[1].1 * image.height() as f32,
        );
        let tl = (tl.0 as u32, tl.1 as u32);
        let br = (br.0 as u32, br.1 as u32);
        [tl, br]
    }

    pub fn preprocess(
        &self,
        image: &DynamicImage,
        template: &DynamicImage,
    ) -> (DynamicImage, DynamicImage) {
        let [tl, br] = self.calc_roi(image);

        let cropped = image.crop_imm(tl.0, tl.1, br.0 - tl.0, br.1 - tl.1);
        // let template = core.get_template(&self.template_filename).unwrap();

        // Color mask
        let (image, template) = (
            mask_image(&cropped, self.color_mask.clone()),
            mask_image(&template, self.color_mask.clone()),
        );
        // masked.save("./masked.png").unwrap();
        // template.save("./masked_template.png").unwrap();

        // Binarize
        let (image, template) = match self.binarize_threshold {
            Some(threshold) => (
                binarize_image(&image, threshold),
                binarize_image(&template, threshold),
            ),
            None => (image.clone(), template),
        };
        // binarized.save("./binarized.png").unwrap();
        // template.save("./binarized_template.png").unwrap();
        (image, template)
    }
}
