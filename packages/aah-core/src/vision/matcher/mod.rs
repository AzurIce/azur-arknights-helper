pub mod best_matcher;
pub mod multi_matcher;
pub mod single_matcher;

const CCORR_THRESHOLD: f32 = 30.0;
const CCORR_NORMED_THRESHOLD: f32 = 0.9;
const CCOEFF_THRESHOLD: f32 = 30.0;
const CCOEFF_NORMED_THRESHOLD: f32 = 0.9;
const SSE_THRESHOLD: f32 = 40.0;
const SSE_NORMED_THRESHOLD: f32 = 0.2;

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
