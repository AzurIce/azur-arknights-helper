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

pub fn normalize_luma32f(
    image: &ImageBuffer<Luma<f32>, Vec<f32>>,
) -> ImageBuffer<Luma<f32>, Vec<f32>> {
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
    ImageBuffer::from_vec(
        image.width(),
        image.height(),
        image
            .as_raw()
            .into_iter()
            .map(|x| (x - min) / (max - min))
            .collect(),
    )
    .unwrap()
}

pub fn luma32f_to_luma8(
    image: &ImageBuffer<Luma<f32>, Vec<f32>>,
) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    ImageBuffer::from_vec(
        image.width(),
        image.height(),
        image.as_raw().iter().map(|x| (x * 255.0) as u8).collect(),
    )
    .unwrap()
}

/// save `image` to `path`,
/// `normalize` indicated whether a linear min-max normalize should be performed
pub fn save_luma32f<P: AsRef<Path>>(
    image: &ImageBuffer<Luma<f32>, Vec<f32>>,
    path: P,
    normalize: bool,
) {
    let image = if normalize {
        normalize_luma32f(image)
    } else {
        image.clone()
    };
    let res_image = luma32f_to_luma8(&image);
    res_image.save(path).unwrap();
}
