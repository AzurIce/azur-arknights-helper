use image::{DynamicImage, GenericImage, Luma, Rgba};

pub fn rgb_to_hsv_v(pixel: &Rgba<u8>) -> u8 {
    let r = pixel[0];
    let g = pixel[1];
    let b = pixel[2];

    let max = r.max(g).max(b);
    // HSV V is simply the max of the RGB values
    max
}

pub fn average_hsv_v(image: &DynamicImage) -> u8 {
    let (sum, count) = image
        .to_rgba8()
        .pixels()
        .map(|p| rgb_to_hsv_v(p))
        .fold((0, 0), |(sum, count), v| (sum + v as u32, count + 1));
    (sum / count) as u8
}

pub fn binarize_image(image: &DynamicImage, threshold: u8) -> DynamicImage {
    let mut image = image.to_luma8();
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let Luma([gray]) = *pixel;

        let binary_value = if gray >= threshold { 255u8 } else { 0u8 };

        *pixel = Luma([binary_value]);
    }
    DynamicImage::ImageLuma8(image)
}

pub fn draw_box(
    image: &mut DynamicImage,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    rgba_u8: [u8; 4],
) {
    // println!("draw box on: {}x{}, box: ({},{}) {}x{}", image.width(), image.height(), x, y, width, height);
    for dx in 0..width {
        let px = x + dx as i32;
        let py1 = y;
        let py2 = y + height as i32;

        if px >= 0 && py1 >= 0 && px < image.width() as i32 && py2 < image.height() as i32 {
            image.put_pixel(px as u32, py1 as u32, Rgba(rgba_u8));
        }
        if px >= 0 && py2 >= 0 && px < image.width() as i32 && py2 < image.height() as i32 {
            image.put_pixel(px as u32, py2 as u32, Rgba(rgba_u8));
        }
    }

    for dy in 0..height {
        let py = y + dy as i32;
        let px1 = x;
        let px2 = x + width as i32;

        if px1 >= 0 && py >= 0 && px1 < image.width() as i32 && py < image.height() as i32 {
            image.put_pixel(px1 as u32, py as u32, Rgba(rgba_u8));
        }
        if px2 >= 0 && py >= 0 && px2 < image.width() as i32 && py < image.height() as i32 {
            image.put_pixel(px2 as u32, py as u32, Rgba(rgba_u8));
        }
    }
    // for dx in 0..width {
    //     for dy in 0..=height {
    //         let px = x + dx as i32;
    //         let py = y + dy as i32;
    //         // 边界检查
    //         if px >= 0 && py >= 0 && px < image.width() as i32 && py < image.height() as i32 {
    //             image.put_pixel(px as u32, py as u32, Rgba(rgba_u8));
    //         }
    //     }
    // }
}

pub fn save_image(image: &DynamicImage, path: &str) {
    let mut path = path.to_string();
    if !path.ends_with(".png") {
        path.push_str(".png")
    }
    image
        .save_with_format(path, image::ImageFormat::Png)
        .expect("failed to save");
}

// pub fn try_init_ocr_engine() -> Result<OcrEngine, Box<dyn Error>> {
//     println!("Initializing ocr engine...");
//     if fs::File::open("text-detection.rten").is_err() {
//         let client = reqwest::blocking::get(
//             "https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten",
//         )?;
//         fs::write("text-detection.rten", client.bytes()?)?;
//     }
//     if fs::File::open("text-recognition.rten").is_err() {
//         let client = reqwest::blocking::get(
//             "https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten",
//         )?;
//         fs::write("text-recognition.rten", client.bytes()?)?;
//     }

//     let detection_model_data = fs::read("text-detection.rten")?;
//     let rec_model_data = fs::read("text-recognition.rten")?;

//     let detection_model = Model::load(&detection_model_data)?;
//     let recognition_model = Model::load(&rec_model_data)?;

//     let engine = OcrEngine::new(OcrEngineParams {
//         detection_model: Some(detection_model),
//         recognition_model: Some(recognition_model),
//         ..Default::default()
//     })?;
//     Ok(engine)
// }
