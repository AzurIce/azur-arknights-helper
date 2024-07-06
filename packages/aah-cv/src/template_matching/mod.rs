pub mod cross_correlation;
pub mod cross_correlation_normed;

use image::{ImageBuffer, Luma};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MatchTemplateMethod {
    CrossCorrelation,
    CrossCorrelationNormed,
}

pub trait Matcher {
    fn match_template(
        &self,
        input: &ImageBuffer<Luma<f32>, Vec<f32>>,
        template: &ImageBuffer<Luma<f32>, Vec<f32>>,
    ) -> ImageBuffer<Luma<f32>, Vec<f32>>;
}
