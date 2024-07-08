use std::path::Path;

use image::{DynamicImage, ImageBuffer, Luma};

pub fn image_mean(image: &ImageBuffer<Luma<f32>, Vec<f32>>) -> f32 {
    let mut sum = 0.0;
    for pixel in image.pixels() {
        sum += pixel[0];
    }
    sum / (image.width() * image.height()) as f32
}

pub fn square_sum(image: &ImageBuffer<Luma<f32>, Vec<f32>>) -> f32 {
    let mut sum = 0.0;
    for pixel in image.pixels() {
        sum += pixel[0] * pixel[0];
    }
    sum
}

pub fn rgb_to_luma(image: &DynamicImage) -> ImageBuffer<Luma<f32>, Vec<f32>> {
    let image = image.to_rgb32f();
    ImageBuffer::from_fn(image.width(), image.height(), |x, y| {
        let [r, g, b] = image.get_pixel(x, y).0;
        Luma([0.299 * r + 0.587 * g + 0.114 * b])
    })
}

/// save `image` to `path`,
/// `normalize` indicated whether a linear min-max normalize should be performed
pub fn save_luma32f<P: AsRef<Path>>(
    image: &ImageBuffer<Luma<f32>, Vec<f32>>,
    path: P,
    normalize: bool,
) {
    let max = image
        .as_raw()
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    let min = image
        .as_raw()
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    let res_image: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_vec(
        image.width(),
        image.height(),
        image
            .as_raw()
            .into_iter()
            .map(|x| {
                if normalize {
                    ((x - min) / (max - min) * 255.0) as u8
                } else {
                    (x * 255.0) as u8
                }
            })
            .collect(),
    )
    .unwrap();
    res_image.save(path).unwrap();
}