use image::{DynamicImage, GenericImage, Luma, Rgba};
use serde::Serialize;

pub mod resource {
    use std::{fs, path::Path};

    use image::DynamicImage;

    /// 从 `{res_path}/resources/templates/1920x1080` 目录中根据文件名称获取模板
    pub fn get_template<S: AsRef<str>, P: AsRef<Path>>(
        template_filename: S,
        res_dir: P,
    ) -> Result<DynamicImage, String> {
        let filename = template_filename.as_ref();
        let res_dir = res_dir.as_ref();

        let path = res_dir.join("templates").join("1920x1080").join(filename);
        let image = image::open(path).map_err(|err| format!("template not found: {err}"))?;
        Ok(image)
    }

    pub fn get_opers<P: AsRef<Path>>(res_dir: P) -> Vec<String> {
        let res_dir = res_dir.as_ref();
        let mut opers: Vec<(u32, String)> = vec![];
        let char_avatars = fs::read_dir(res_dir.join("avatars").join("char")).unwrap();
        for char_avatar in char_avatars {
            let char_avatar = char_avatar.unwrap();
            if char_avatar.file_type().unwrap().is_file() {
                continue;
            }
            let mut name = fs::read_dir(char_avatar.path()).unwrap();
            let f = name.next().unwrap().unwrap();
            let name = f.file_name().into_string().unwrap();
            let name = name.split('_').collect::<Vec<&str>>()[0];
            let name = name.split('.').collect::<Vec<&str>>()[0];
            let num = char_avatar.file_name().into_string().unwrap();
            opers.push((num.parse().unwrap(), name.to_string()));
        }
        opers
            .into_iter()
            .map(|(num, name)| format!("char_{num:03}_{name}.png"))
            .collect()
    }

    /// 输入干员列表和资源路径，以 `Vec<(名,头像)>` 形式返回这些干员的所有头像列表
    pub fn get_opers_avatars<S: AsRef<str>, P: AsRef<Path>>(
        opers: Vec<S>,
        res_dir: P,
    ) -> Result<Vec<(String, DynamicImage)>, String> {
        let res_dir = res_dir.as_ref();
        let oper_images = opers
            .iter()
            .map(|s| {
                let s = s.as_ref();
                get_oper_avatars(s, res_dir).map(|imgs| {
                    imgs.into_iter()
                        .map(|img| (s.to_string(), img))
                        .collect::<Vec<(String, DynamicImage)>>()
                })
            })
            .collect::<Result<Vec<Vec<(String, DynamicImage)>>, String>>()?;
        let oper_images = oper_images
            .into_iter()
            .flatten()
            .collect::<Vec<(String, DynamicImage)>>();
        Ok(oper_images)
    }

    /// 从 `<res_dir>/avatars/` 获取指定角色的所有头像图片
    pub fn get_oper_avatars<S: AsRef<str>, P: AsRef<Path>>(
        oper: S,
        res_dir: P,
    ) -> Result<Vec<DynamicImage>, String> {
        let oper = oper.as_ref();
        let res_dir = res_dir.as_ref();

        // println!("{:?}", oper);
        let components = oper.split("_").collect::<Vec<&str>>();
        let path = res_dir.join("avatars").join("char").join(components[1]);
        // println!("{:?}", path);

        let mut images = vec![];
        for f in fs::read_dir(path).unwrap() {
            let f = f.unwrap();
            println!("{:?}", f.path());
            let image = image::open(f.path()).map_err(|err| format!("avatar not found: {err}"))?;
            // let image = image.crop_imm(30, 20, 100, 120);
            images.push(image);
        }
        Ok(images)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

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
    for (_, _, pixel) in image.enumerate_pixels_mut() {
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
