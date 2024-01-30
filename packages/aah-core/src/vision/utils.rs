use image::DynamicImage;

pub fn save_image(image: &DynamicImage, path: &str) {
    let mut path = path.to_string();
    if !path.ends_with(".png") {
        path.push_str(".png")
    }
    image.save_with_format(path, image::ImageFormat::Png).expect("failed to save");
}