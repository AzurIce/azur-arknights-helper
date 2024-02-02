use fft2d::slice::{fft_2d, ifft_2d};
use fftconvolve::{fftconvolve, fftcorrelate};
use image::{GrayImage, Luma};
use imageproc::template_matching::Extremes;
use ndarray::{s, Array, Array2, Zip};
use nshare::RefNdarray2;
use rustfft::{num_complex::Complex, Fft, FftDirection, FftPlanner};

#[cfg(test)]
mod test {
    use std::time::Instant;

    use fft2d::slice::{fft_2d, ifft_2d};
    use image::{DynamicImage, GrayImage, ImageBuffer};
    use ndarray::Array2;
    use nshare::ToNdarray2;
    use rustfft::{num_complex::Complex, FftPlanner};

    use super::convolve_dft;

    #[test]
    fn test_fft() {
        let x = (1..=3).collect::<Vec<u8>>();
        println!("Original: {:?}", x);
        let mut x = x
            .into_iter()
            .map(|x| Complex::new(x as f64, 0.0))
            .collect::<Vec<_>>();
        println!("Original to Complex: {:?}", x);

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(3);
        let inv_fft = planner.plan_fft_inverse(3);

        fft.process(&mut x);
        println!("fft: {:?}", x);

        inv_fft.process(&mut x);
        println!("inv_fft: {:?}", x)
    }

    #[test]
    fn test_image_fft() {
        let x = (1..=36).collect::<Vec<u8>>();
        let image = GrayImage::from_raw(6, 6, x).unwrap().into_ndarray2();
        let image = image.map(|&x| x as f64);
        println!("origin: {:?}", image);
        let mut x = image
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect::<Vec<Complex<f64>>>();

        fft_2d(image.dim().0, image.dim().1, &mut x);
        println!("fft (len = {}): {:?}", x.len(), x);

        // ifft_2d(image.dim().0, image.dim().1, &mut x);
        // let res = x.iter().map(|&x| {
        //     x.re.round() / image.dim().0 as f64 / image.dim().1 as f64
        // }).collect::<Vec<f64>>();
        // println!("inv_fft (len = {}): {:?}", res.len(), res)
        let x = (1..=16).collect::<Vec<u8>>();
        let image = GrayImage::from_raw(4, 4, x).unwrap().into_ndarray2();
        let image = image.map(|&x| x as f64);
        println!("origin: {:?}", image);
        let mut x = image
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect::<Vec<Complex<f64>>>();

        fft_2d(image.dim().0, image.dim().1, &mut x);
        println!("fft (len = {}): {:?}", x.len(), x);
    }

    #[test]
    fn test_convolve_dft() {
        let x = (1..=36).collect::<Vec<u8>>();
        let image = GrayImage::from_raw(6, 6, x).unwrap().into_ndarray2();
        let image = image.map(|&x| x as f32);
        println!("image: {:?}", image);
        let template = Array2::from_shape_simple_fn((2, 2), || 1.0 / 4.0);
        println!("template: {:?}", template);

        let res = convolve_dft(&image, &template);
        println!("res: {:?}", res);
    }

    #[test]
    fn test_template_match() {
        /*
        imageproc: Extremes { max_value: 348514530.0, min_value: 108662460.0, max_value_location: (147, 0), min_value_location: (137, 288) }, cost: 105563
        aah-cv: Extremes { max_value: 5359.685, min_value: 1671.0929, max_value_location: (147, 0), min_value_location: (137, 288) }, cost: 1037
        */
        let image = image::open("./test/image.png").unwrap();
        let template = image::open("./test/template.png").unwrap();

        let start = Instant::now();
        let image_luma8 = image.to_luma8();
        let template_luma8 = template.to_luma8();
        let res = imageproc::template_matching::match_template(&image_luma8, &template_luma8, imageproc::template_matching::MatchTemplateMethod::CrossCorrelation);
        let res = imageproc::template_matching::find_extremes(&res);
        println!("imageproc: {:?}, cost: {}", res, start.elapsed().as_millis());

        let start = Instant::now();
        let image_luma32f = image.to_luma32f();
        let template_luma32f = template.to_luma32f();
        let res = super::match_template(&image_luma32f.into_ndarray2(), &template_luma32f.into_ndarray2());
        let res = super::find_extremes(&res);
        println!("aah-cv: {:?}, cost: {}", res, start.elapsed().as_millis());
    }
}

pub fn match_template(image: &Array2<f32>, kernel: &Array2<f32>) -> Array2<f32> {
    // let conv = convolve_dft(image, kernel);
    let conv = fftcorrelate(image, kernel, fftconvolve::Mode::Valid).unwrap();
    conv
}

pub fn find_extremes(input: &Array2<f32>) -> Extremes<f32> {
    let mut min_value = f32::MAX;
    let mut min_value_location = (0, 0);
    let mut max_value = f32::MIN;
    let mut max_value_location = (0, 0);

    input.iter().enumerate().for_each(|(idx, &v)| {
        let y = idx / input.dim().1;
        let x = idx % input.dim().1;

        if v < min_value {
            min_value = v;
            min_value_location = (x, y);
        }

        if v > max_value {
            max_value = v;
            max_value_location = (x, y);
        }
    });

    Extremes {
        min_value,
        max_value,
        min_value_location: (min_value_location.0 as u32, min_value_location.1 as u32),
        max_value_location: (max_value_location.0 as u32, max_value_location.1 as u32),
    }
}

fn convolve_dft(image: &Array2<f32>, kernel: &Array2<f32>) -> Array2<f32> {
    assert!(image.len() >= kernel.len());

    fftconvolve(image, kernel, fftconvolve::Mode::Valid).unwrap()
    // let (image_width, image_height) = image.dim();
    // let (kernel_width, kernel_height) = kernel.dim();

    // let mut image = image.iter().map(|&x| Complex::new(x as f64, 0.0)).collect::<Vec<Complex<f64>>>();
    // fft_2d(image_width, image_height, &mut image);
    // let mut kernel = kernel.iter().map(|&x| Complex::new(x as f64, 0.0)).collect::<Vec<Complex<f64>>>();
    // fft_2d(kernel_width, kernel_height, &mut kernel);

    // let mut res = image.iter().zip(kernel.iter()).map(|(&x, &y)| {
    //     x * y
    // }).collect::<Vec<Complex<f64>>>();
    // ifft_2d(image_width, image_height, &mut res);
    // Array2::from_shape_fn((image_height, image_width), |(y, x)| {
    //     res[y * image_width + x].re.round() as f32
    // })
}
