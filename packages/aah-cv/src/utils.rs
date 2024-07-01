use image::{DynamicImage, GenericImageView, ImageBuffer, Luma};

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
