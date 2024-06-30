pub mod best_matcher;
pub mod multi_matcher;

use std::error::Error;

use image::DynamicImage;
use rten_tensor::{NdTensorBase, NdTensorView};
// use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};

const THRESHOLD: f32 = 100.0;
const SSE_THRESHOLD: f32 = 40.0;

pub fn convert_image_to_ten(
    image: DynamicImage,
) -> Result<NdTensorBase<f32, Vec<f32>, 3>, Box<dyn Error>> {
    let image = image.into_rgb8();
    let (width, height) = image.dimensions();
    let layout = image.sample_layout();

    let chw_tensor = NdTensorView::from_slice(
        image.as_raw().as_slice(),
        [height as usize, width as usize, 3],
        Some([
            layout.height_stride,
            layout.width_stride,
            layout.channel_stride,
        ]),
    )
    .map_err(|err| format!("failed to convert image to tensorL {:?}", err))?
    .permuted([2, 0, 1]) // HWC => CHW
    .to_tensor() // Make tensor contiguous, which makes `map` faster
    .map(|x| *x as f32 / 255.); // Rescale from [0, 255] to [0, 1]
    Ok(chw_tensor)
}

#[cfg(test)]
pub mod test {
    use image::DynamicImage;
    use std::path::Path;

    fn get_image<P: AsRef<Path>>(path: P) -> Result<DynamicImage, String> {
        image::open(path).map_err(|err| format!("failed to open image: {:?}", err))
    }

    pub fn get_device_image<P: AsRef<Path>>(
        device: Device,
        filename: P,
    ) -> Result<DynamicImage, String> {
        let templates_path = Path::new("../../resources/templates");
        let image_path = templates_path.join(device.folder_name());
        get_image(image_path.join(filename))
    }

    fn get_template<P: AsRef<Path>>(filename: P) -> Result<DynamicImage, String> {
        let templates_path = Path::new("../../resources/templates");
        let template_path = templates_path.join("1920x1080");
        get_image(template_path.join(filename))
    }

    pub fn get_device_template_prepared<P: AsRef<Path>>(
        device: Device,
        filename: P,
    ) -> Result<DynamicImage, String> {
        let orinigal_template = get_template(filename)?;
        let template = orinigal_template;
        let template = {
            let new_width = (template.width() as f32 * device.factor()) as u32;
            let new_height = (template.height() as f32 * device.factor()) as u32;

            DynamicImage::ImageRgba8(image::imageops::resize(
                &template,
                new_width,
                new_height,
                image::imageops::FilterType::Triangle,
            ))
        };
        Ok(template)
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Device {
        MUMU,
        P40Pro,
    }

    impl Device {
        pub fn factor(&self) -> f32 {
            match self {
                Device::MUMU => 1.0,
                Device::P40Pro => 0.83,
            }
        }
        pub fn folder_name(&self) -> &str {
            match self {
                Device::MUMU => "MUMU-1920x1080",
                Device::P40Pro => "P40 Pro-2640x1200",
            }
        }
    }
}
