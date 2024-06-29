use std::{error::Error, fs};

use image::DynamicImage;
// use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;

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
