use std::f32::consts::PI;

use ndarray::{Array1, Array2, Axis};

use crate::{arknights::Aah, task::Runnable};

#[cfg(test)]
mod test {
    #[test]
    fn test_depot_analyzer() {
        // let controller = MiniTouchController::connect("127.0.0.1:16384").unwrap();

        // let mut analyzer = DepotAnalyzer::new();
        // let res = analyzer.analyze(&controller).unwrap();
        // println!("{:?}", res);
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

impl Runnable<Aah> for DepotAnalyzer {
    type Res = DepotAnalyzerOutput;

    fn run(&self, aah: &Aah) -> anyhow::Result<Self::Res> {
        let crop_height = 128 + 30;
        let x_period = 312;
        let y_period = 380;

        let mut screen = aah
            .controller
            .screencap_scaled()
            .map_err(|err| anyhow::anyhow!("{:?}", err))?;

        let screen = screen.crop(
            0,
            crop_height,
            screen.width(),
            screen.height() - crop_height,
        );

        let gray = screen.to_luma32f();
        let gray_arr2: Array2<f32> =
            Array2::from_shape_fn((gray.height() as usize, gray.width() as usize), |(y, x)| {
                gray.get_pixel(x as u32, y as u32).0[0]
            });

        let mut hist_x = gray_arr2
            .sum_axis(Axis(0))
            .map(|v| *v as f32 / gray_arr2.shape()[1] as f32);

        // 归一化
        let min_val = hist_x.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_val = hist_x.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        hist_x.mapv_inplace(|x| (x - min_val) / (max_val - min_val));

        let sin = Array1::from_shape_fn([screen.width() as usize], |x| {
            (2.0 * PI / x_period as f32 * x as f32).sin()
        });
        let cos = Array1::from_shape_fn([screen.width() as usize], |x| {
            (2.0 * PI / x_period as f32 * x as f32).cos()
        });

        let hist_sin = sin * hist_x.clone();
        let hist_cos = cos * hist_x;

        let s_v = hist_sin.sum();
        let c_v = hist_cos.sum();

        let phase = s_v.atan2(c_v);
        // println!("phase: {}", phase);
        let x_first = phase / (2.0 * PI) * x_period as f32 + (x_period / 2) as f32;
        // println!("x_first: {}", x_first);

        let res = DepotAnalyzerOutput {};
        Ok(res)
    }
}
