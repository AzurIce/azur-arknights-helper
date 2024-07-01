use image::{ImageBuffer, Luma};


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