use image::{GrayImage, Luma};
use ndarray::{Array, Array2, s, Zip};
use rustfft::{num_complex::Complex, Fft, FftDirection, FftPlanner};
use nshare::RefNdarray2;

#[cfg(test)]
mod test {
    use image::{DynamicImage, GrayImage, ImageBuffer};
    use ndarray::Array2;
    use nshare::ToNdarray2;
    use rustfft::{num_complex::Complex, FftPlanner};

    use super::convolve_dft;

    #[test]
    fn test_fft() {
        let x = (1..=3).collect::<Vec<u8>>();
        println!("Original: {:?}", x);
        let mut x = x.into_iter().map(|x| Complex::new(x as f64, 0.0)).collect::<Vec<_>>();
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
    fn test_convolve_dft() {
        let x = (1..=36).collect::<Vec<u8>>();
        let image = GrayImage::from_raw(6, 6, x).unwrap().into_ndarray2();
        let image = image.map(|&x| x as f32);
        println!("image: {:?}", image);
        let template = Array2::from_shape_simple_fn((5, 5), ||0.25f32);
        println!("template: {:?}", template);

        let res = convolve_dft(&image, &template);
        println!("res: {:?}", res);
    }
}

fn convolve_dft(image: &Array2<f32>, kernel: &Array2<f32>) -> Array2<f32> {
    assert!(image.len() >= kernel.len());

    let (image_width, image_height) = image.dim();
    let (kernel_width, kernel_height) = kernel.dim();
    let fft_width = (image_width + kernel_width - 1).next_power_of_two() as usize;
    let fft_height = (image_height + kernel_height - 1).next_power_of_two() as usize;

    let mut planner = FftPlanner::new();
    let fft_image = planner.plan_fft_forward(fft_width);
    let fft_kernel = planner.plan_fft_forward(fft_width);

    // Create complex arrays for image and kernel
    let mut complex_image: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); fft_width * fft_height];
    let mut complex_kernel: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); fft_width * fft_height];

    // Copy image and kernel data to complex arrays
    for y in 0..image_height {
        for x in 0..image_width {
            complex_image[y as usize * fft_width + x as usize].re = *image.get((x, y)).unwrap();
        }
    }

    for y in 0..kernel_height {
        for x in 0..kernel_width {
            complex_kernel[y as usize * fft_width + x as usize].re = *kernel.get((x, y)).unwrap();
        }
    }

    // Perform FFT on image and kernel
    fft_image.process(&mut complex_image);
    fft_kernel.process(&mut complex_kernel);

    // Element-wise multiplication in frequency domain

    for i in 0..fft_width * fft_height {
        complex_image[i] *= complex_kernel[i];
    }

    // Create FFT planner for inverse FFT
    let mut planner_inverse = FftPlanner::new();
    let fft_inverse = planner_inverse.plan_fft_inverse(fft_width);

    // Perform inverse FFT on the result
    fft_inverse.process(&mut complex_image);

    let res_width = image_width - kernel_width + 1;
    let res_height = image_height - kernel_height + 1;
    let result_array = Array2::from_shape_fn((res_height, res_width), |(y, x)| {
        let index = y * fft_width + x;
        complex_image[index].re.round()
    });

    result_array
}