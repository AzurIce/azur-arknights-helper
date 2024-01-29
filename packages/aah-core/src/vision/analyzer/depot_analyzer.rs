use std::f32::consts::PI;

use image::{DynamicImage, GenericImage, GenericImageView, Luma, Pixel};
use ndarray::{Array2, Axis, Array, Array1};

use crate::{
    adb::{AdbTcpStream, Device},
    controller::Controller,
};

use super::Analyzer;

#[cfg(test)]
mod test {
    use super::*;
    use crate::{controller::MiniTouchController, vision::analyzer::Analyzer};

    #[test]
    fn test_depot_analyzer() {
        let controller = MiniTouchController::connect("127.0.0.1:16384").unwrap();

        let mut analyzer = DepotAnalyzer::new();
        let res = analyzer.analyze(&controller).unwrap();
        println!("{:?}", res);
    }
}

#[derive(Debug)]
pub struct DepotAnalyzerOutput {}

pub struct DepotAnalyzer {}

impl DepotAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
}

fn save_hist_x(hist_x: &Array1<f32>, name: &str) {
    let hist_image = image::GrayImage::from_fn(hist_x.len() as u32, 100, |x, y| {
        Luma([(*hist_x.get([x as usize]).unwrap() * 255.0) as u8])
    });
    hist_image
        .save_with_format(name, image::ImageFormat::Png)
        .expect("failed to save pic");
}

impl Analyzer for DepotAnalyzer {
    type Output = DepotAnalyzerOutput;

    fn analyze(&mut self, controller: &impl Controller) -> Result<Self::Output, String> {
        let crop_height = 128 + 30;
        let x_period = 312;
        let y_period = 380;

        let mut screen = controller.screencap().map_err(|err| format!("{:?}", err))?;
        screen
            .save_with_format("./tmp_original.png", image::ImageFormat::Png)
            .expect("failed to save pic");

        let screen = screen.crop(0, crop_height, screen.width(), screen.height() - crop_height);

        let gray = screen.to_luma8();
        gray.save_with_format("./tmp_gray.png", image::ImageFormat::Png)
            .expect("failed to save pic");
        let gray = screen.to_luma32f();
        let gray_arr2: Array2<f32> =
            Array2::from_shape_fn((gray.height() as usize, gray.width() as usize), |(y, x)| {
                gray.get_pixel(x as u32, y as u32).0[0]
            });

        // let gray_arr2 = cvt_color_rgb2gray(&screen);
        // Assuming hist_x is a 1D ndarray representing the horizontal histogram
        let mut hist_x = gray_arr2
            .sum_axis(Axis(0))
            .map(|v| *v as f32 / gray_arr2.shape()[1] as f32);
        println!("{:?}", hist_x);
        save_hist_x(&hist_x, "tmp_hist_x.png");

        // 归一化
        let min_val = hist_x.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_val = hist_x.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        hist_x.mapv_inplace(|x| (x - min_val) / (max_val - min_val));
        println!("{:?}", hist_x);
        save_hist_x(&hist_x, "tmp_hist_x_normalized.png");

        let sin = Array1::from_shape_fn([screen.width() as usize], |x| {
            (2.0 * PI / x_period as f32 * x as f32).sin()
        });
        let cos = Array1::from_shape_fn([screen.width() as usize], |x| {
            (2.0 * PI / x_period as f32 * x as f32).cos()
        });

        let hist_sin = sin * hist_x.clone();
        let hist_cos = cos * hist_x;
        save_hist_x(&hist_sin, "tmp_hist_sin.png");
        save_hist_x(&hist_cos, "tmp_hist_cos.png");

        let s_v = hist_sin.sum();
        let c_v = hist_cos.sum();
        println!("sin: {}, cos: {}", s_v, c_v);

        let phase = s_v.atan2(c_v);
        println!("phase: {}", phase);
        let x_first = phase / (2.0 * PI) * x_period as f32 + (x_period / 2) as f32;
        println!("x_first: {}", x_first);

        let res = DepotAnalyzerOutput {};
        Ok(res)
    }
}
