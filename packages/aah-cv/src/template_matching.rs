use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

use fft2d::slice::{fft_2d, ifft_2d};
use fftconvolve::{fftconvolve, fftcorrelate};
use image::{GrayImage, Luma};
use imageproc::{integral_image::integral_image, template_matching::Extremes};
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
        imageproc(CrossCorrelation): Extremes { max_value: 348514530.0, min_value: 108662460.0, max_value_location: (147, 0), min_value_location: (137, 288) }, cost: 108810
        imageproc(CrossCorrelationNormalized): Extremes { max_value: 0.9999335, min_value: 0.5512544, max_value_location: (88, 227), min_value_location: (140, 316) }, cost: 113587
        imageproc(SumOfSquaredErrors): Extremes { max_value: 411913200.0, min_value: 38708.0, max_value_location: (343, 81), min_value_location: (88, 227) }, cost: 189774
        aah-cv: Extremes { max_value: 5359.685, min_value: 1671.0929, max_value_location: (147, 0), min_value_location: (137, 288) }, cost: 1098
        */
        let image = image::open("./test/image.png").unwrap();
        let template = image::open("./test/template.png").unwrap();

        let image_luma8 = image.to_luma8();
        let template_luma8 = template.to_luma8();
        let image_luma32f = image.to_luma32f();
        let template_luma32f = template.to_luma32f();

        // let start = Instant::now();
        // let res = imageproc::template_matching::match_template(
        //     &image_luma8,
        //     &template_luma8,
        //     imageproc::template_matching::MatchTemplateMethod::CrossCorrelation,
        // );
        // let res = imageproc::template_matching::find_extremes(&res);
        // println!(
        //     "imageproc(CrossCorrelation): {:?}, cost: {}s",
        //     res,
        //     start.elapsed().as_secs_f32()
        // );

        let start = Instant::now();
        let res = imageproc::template_matching::match_template(
            &image_luma8,
            &template_luma8,
            imageproc::template_matching::MatchTemplateMethod::CrossCorrelationNormalized,
        );
        let res = imageproc::template_matching::find_extremes(&res);
        println!(
            "imageproc(CrossCorrelationNormalized): {:?}, cost: {}s",
            res,
            start.elapsed().as_secs_f32()
        );

        // let start = Instant::now();
        // let res = imageproc::template_matching::match_template(
        //     &image_luma8,
        //     &template_luma8,
        //     imageproc::template_matching::MatchTemplateMethod::SumOfSquaredErrors,
        // );
        // let res = imageproc::template_matching::find_extremes(&res);
        // println!(
        //     "imageproc(SumOfSquaredErrors): {:?}, cost: {}s",
        //     res,
        //     start.elapsed().as_secs_f32()
        // );

        let start = Instant::now();
        let res = super::match_template(
            &image_luma32f.into_ndarray2(),
            &template_luma32f.into_ndarray2(),
        );
        let res = super::find_extremes(&res);
        println!("aah-cv: {:?}, cost: {}s", res, start.elapsed().as_secs_f32());
    }

    use super::*;

    #[test]
    fn test_integral() {
        let mat = Array2::ones((5, 5));
        let integral = integral_arr2f32(&mat);
        println!("{:?}", integral);
        assert_eq!(
            integral,
            Array2::from_shape_fn((5, 5), |(y, x)| { (x as f32 + 1.0) * (y as f32 + 1.0) })
        );
        let res = subsum_from_integral_arrf32(&integral, 2, 2, 3, 3);
        assert_eq!(res, 9.0);
        let res = subsum_from_integral_arrf32(&integral, 0, 2, 2, 2);
        assert_eq!(res, 4.0);
        let res = subsum_from_integral_arrf32(&integral, 0, 0, 2, 2);
        assert_eq!(res, 4.0);
    }

    #[test]
    fn test_integral_squared() {
        let mat = Array2::ones((5, 5));
        let integral = integral_square_arr2f32(&mat);
        println!("{:?}", integral);
        assert_eq!(
            integral,
            Array2::from_shape_fn((5, 5), |(y, x)| { (x as f32 + 1.0) * (y as f32 + 1.0) })
        );
        let mat = Array2::from_shape_fn((5, 5), |(y, x)| y as f32 * 5.0 + x as f32);
        let mat_squared = mat.map(|&x| x * x);
        let res_integral = integral_arr2f32(&mat_squared);
        let integral = integral_square_arr2f32(&mat);
        println!("{:?}", integral);
        assert_eq!(integral, res_integral);
    }
}

pub fn match_template(image: &Array2<f32>, kernel: &Array2<f32>) -> Array2<f32> {
    // let conv = convolve_dft(image, kernel);
    let integral_image = integral_square_arr2f32(image);
    let suqared_sum_kernal = kernel.map(|x| x * x).sum();

    let mut res = fftcorrelate(image, kernel, fftconvolve::Mode::Valid).unwrap();

    let (y_image, x_image) = image.dim();
    let (y_kernel, x_kernel) = kernel.dim();
    let (y_len, x_len) = (y_image - y_kernel + 1, x_image - x_kernel + 1);
    for x in 0..x_len {
        for y in 0..y_len {
            res.get_mut((y, x)).unwrap().div_assign(
                (suqared_sum_kernal
                    * subsum_from_integral_arrf32(&integral_image, x, y, x_kernel, y_kernel))
                .sqrt(),
            )
        }
    }

    res
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

pub fn integral_arr2f32(mat: &Array2<f32>) -> Array2<f32> {
    let (x_len, y_len) = mat.dim();

    let mut res = mat.clone();
    for cur_y in 0..x_len {
        for cur_x in 0..y_len {
            if cur_x > 0 && cur_y > 0 {
                let v = res[[cur_y - 1, cur_x]];
                res.get_mut((cur_y, cur_x)).unwrap().add_assign(v);
                let v = res[[cur_y, cur_x - 1]];
                res.get_mut((cur_y, cur_x)).unwrap().add_assign(v);
                let v = res[[cur_y - 1, cur_x - 1]];
                res.get_mut((cur_y, cur_x)).unwrap().sub_assign(v);
            } else {
                if cur_y > 0 {
                    let v = res[[cur_y - 1, cur_x]];
                    res.get_mut((cur_y, cur_x)).unwrap().add_assign(v);
                }
                if cur_x > 0 {
                    let v = res[[cur_y, cur_x - 1]];
                    res.get_mut((cur_y, cur_x)).unwrap().add_assign(v);
                }
            }
        }
    }
    res
}

pub fn integral_square_arr2f32(mat: &Array2<f32>) -> Array2<f32> {
    let (x_len, y_len) = mat.dim();

    let mut res = mat.clone();
    for cur_y in 0..x_len {
        for cur_x in 0..y_len {
            let v = res[[cur_y, cur_x]];
            res.get_mut((cur_y, cur_x)).unwrap().mul_assign(v);
            if cur_x > 0 && cur_y > 0 {
                let v = res[[cur_y - 1, cur_x]];
                res.get_mut((cur_y, cur_x)).unwrap().add_assign(v);
                let v = res[[cur_y, cur_x - 1]];
                res.get_mut((cur_y, cur_x)).unwrap().add_assign(v);
                let v = res[[cur_y - 1, cur_x - 1]];
                res.get_mut((cur_y, cur_x)).unwrap().sub_assign(v);
            } else {
                if cur_y > 0 {
                    let v = res[[cur_y - 1, cur_x]];
                    res.get_mut((cur_y, cur_x)).unwrap().add_assign(v);
                }
                if cur_x > 0 {
                    let v = res[[cur_y, cur_x - 1]];
                    res.get_mut((cur_y, cur_x)).unwrap().add_assign(v);
                }
            }
        }
    }
    res
}

pub fn subsum_from_integral_arrf32(
    integral_mat: &Array2<f32>,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> f32 {
    let right = x + width - 1;
    let bottom = y + height - 1;
    let res = integral_mat[[bottom, right]];
    if x > 0 && y > 0 {
        res + integral_mat[[y - 1, x - 1]]
            - integral_mat[[bottom, x - 1]]
            - integral_mat[[y - 1, right]]
    } else {
        if x > 0 {
            res - integral_mat[[bottom, x - 1]]
        } else if y > 0 {
            res - integral_mat[[y - 1, right]]
        } else {
            res
        }
    }
}

pub fn square_sum_arr2f32(mat: &Array2<f32>) -> f32 {
    mat.iter().map(|&p| p * p).sum()
}
