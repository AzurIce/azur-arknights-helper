use std::{
    error::Error,
    ops::{AddAssign, SubAssign},
    time::Instant,
};

use fftconvolve::{fftcorrelate, Mode};
use imageproc::template_matching::Extremes;
use ndarray::{
    Array, Array1, Array2, ArrayBase, AssignElem, Axis, Data, DataMut, Dimension, OwnedRepr, Slice,
};
use rustfft::{
    num_complex::Complex,
    num_traits::{FromPrimitive, Zero},
    FftNum,
};

#[cfg(test)]
mod test {
    use std::{path::Path, time::Instant};

    use fft2d::slice::fft_2d;
    use image::{GrayImage, Luma};
    use ndarray::Array2;
    use nshare::ToNdarray2;
    use rustfft::{num_complex::Complex, FftPlanner};

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
        let t = Instant::now();
        let image = GrayImage::from_fn(2560, 1440, |_, _| Luma([128])).into_ndarray2();
        // let image = GrayImage::from_fn(1440, 2560, |_, _| Luma([128])).into_ndarray2();
        let image = image.map(|&x| x as f64);
        // println!("origin: {:?}", image);
        let mut x = image
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect::<Vec<Complex<f64>>>();

        fft_2d(image.dim().0, image.dim().1, &mut x);
        // println!("fft (len = {}): {:?}", x.len(), x);

        // let image = GrayImage::from_fn(512, 512, |_, _| Luma([128])).into_ndarray2();
        // let image = image.map(|&x| x as f64);
        // println!("origin: {:?}", image);
        // let mut x = image
        //     .iter()
        //     .map(|&x| Complex::new(x, 0.0))
        //     .collect::<Vec<Complex<f64>>>();

        // fft_2d(image.dim().0, image.dim().1, &mut x);
        // println!("fft (len = {}): {:?}", x.len(), x);
        println!("{:?}", t.elapsed());
    }

    fn test_template_match_with_image_and_template(image: &str, template: &str) {
        println!("matching {} {}...", image, template);
        let image = image::open(Path::new("./test").join(image)).unwrap();
        let template = image::open(Path::new("./test").join(template)).unwrap();
        println!(
            "image: {}x{}, template: {}x{}",
            image.width(),
            image.height(),
            template.width(),
            template.height()
        );

        let image_luma32f = image.to_luma32f();
        let template_luma32f = template.to_luma32f();
        // let image_luma8 = image.to_luma8();
        // let template_luma8 = template.to_luma8();

        let start = Instant::now();
        let res = super::match_template(
            &image_luma32f.into_ndarray2(),
            &template_luma32f.into_ndarray2(),
        );
        let res = super::find_extremes(&res.map(|&x| x as f32));
        println!(
            "aah-cv: {:?}, cost: {}s",
            res,
            start.elapsed().as_secs_f32()
        );
    }

    #[test]
    fn test_template_match() {
        // test_template_match_with_image_and_template("image.png", "template.png");
        // test_template_match_with_image_and_template("main.png", "EnterMissionMistCity.png");
        test_template_match_with_image_and_template("start.png", "start_btn.png");
    }

    use super::*;

    #[test]
    fn test_integral() {
        let mat = Array2::ones((5, 5));
        let integral = integral_arr2(&mat);
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
}

/// Pad the edges of an array with zeros.
///
/// `pad_width` specifies the length of the padding at the beginning
/// and end of each axis.
///
/// Returns a Result. Errors if arr.ndim() != pad_width.len().
fn pad_with_zeros<A, S, D>(
    arr: &ArrayBase<S, D>,
    pad_width: Vec<[usize; 2]>,
) -> Result<Array<A, D>, Box<dyn Error>>
where
    A: FftNum,
    S: Data<Elem = A>,
    D: Dimension,
{
    if arr.ndim() != pad_width.len() {
        return Err("arr.ndim() != pad_width.len()".into());
    }

    // Compute shape of final padded array.
    let mut padded_shape = arr.raw_dim();
    for (ax, (&ax_len, &[pad_lo, pad_hi])) in arr.shape().iter().zip(&pad_width).enumerate() {
        padded_shape[ax] = ax_len + pad_lo + pad_hi;
    }

    let mut padded = Array::zeros(padded_shape);
    let padded_dim = padded.raw_dim();
    {
        // Select portion of padded array that needs to be copied from the
        // original array.
        let mut orig_portion = padded.view_mut();
        for (ax, &[pad_lo, pad_hi]) in pad_width.iter().enumerate() {
            orig_portion.slice_axis_inplace(
                Axis(ax),
                Slice::from(pad_lo as isize..padded_dim[ax] as isize - (pad_hi as isize)),
            );
        }
        // Copy the data from the original array.
        orig_portion.assign(arr);
    }
    Ok(padded)
}

/// Generates a Vec<[usize; 2]> specifying how much to pad each axis.
fn generate_pad_vector<A, S, D>(arr: &ArrayBase<S, D>, shape: &[usize]) -> Vec<[usize; 2]>
where
    A: FftNum,
    S: Data<Elem = A>,
    D: Dimension,
{
    arr.shape()
        .into_iter()
        .zip(shape.iter())
        .map(|(arr_size, new_size)| [0, new_size - arr_size])
        .collect()
}

use easyfft::dyn_size::{DynFftMut, DynIfftMut};

/// Convolve two N-dimensional arrays using FFT.
pub fn fftconvolve<A, S, D>(
    in1: &ArrayBase<S, D>,
    in2: &ArrayBase<S, D>,
    mode: Mode,
) -> Result<ArrayBase<OwnedRepr<A>, D>, Box<dyn Error>>
where
    A: FftNum + FromPrimitive + Default,
    S: Data<Elem = A>,
    D: Dimension,
{
    // check that arrays have the same number of dimensions
    if in1.ndim() != in2.ndim() {
        return Err("Input arrays must have the same number of dimensions.".into());
    }

    // Pad the arrays to the next power of 2.
    let mut shape = in1.shape().to_vec();
    let s1 = Array::from_vec(
        in1.shape()
            .into_iter()
            .map(|a| *a as isize)
            .collect::<Vec<_>>(),
    );
    let s2 = Array::from_vec(
        in2.shape()
            .into_iter()
            .map(|a| *a as isize)
            .collect::<Vec<_>>(),
    );
    for (s, s_other) in shape.iter_mut().zip(in2.shape().iter()) {
        *s = *s + *s_other - 1;
    }
    let in1 = pad_with_zeros(in1, generate_pad_vector(&in1, shape.as_slice()))?;
    let in2 = pad_with_zeros(in2, generate_pad_vector(&in2, shape.as_slice()))?;

    // multiple values in shape together to get total size
    let total_size = shape.iter().fold(1, |acc, x| acc * x);

    let mut in1 = in1.mapv(|x| Complex::new(x, Zero::zero()));
    let mut in2 = in2.mapv(|x| Complex::new(x, Zero::zero()));
    in1.as_slice_mut().unwrap().fft_mut();
    in2.as_slice_mut().unwrap().fft_mut();

    // Multiply the FFTs.
    let mut out = in1 * in2;

    out.as_slice_mut().unwrap().ifft_mut();

    // Return the real part of the result. Note normalise by 1/total_size
    let total_size = A::from_usize(total_size).unwrap();

    match mode {
        Mode::Full => {
            let out = out.mapv(|x| x.re / total_size);
            Ok(out)
        }
        Mode::Same => {
            let mut out = out.mapv(|x| x.re / total_size);
            centred(&mut out, s1);
            Ok(out)
        }
        Mode::Valid => {
            let mut out = out.mapv(|x| x.re / total_size);
            centred(&mut out, s1 - s2 + 1);
            Ok(out)
        }
    }
}

fn centred<S, D>(arr: &mut ArrayBase<S, D>, s1: Array1<isize>)
where
    S: DataMut,
    D: Dimension,
{
    let out_shape = Array::from_vec(
        arr.shape()
            .into_iter()
            .map(|a| *a as isize)
            .collect::<Vec<_>>(),
    );
    let startind = (out_shape.to_owned() - s1.to_owned()) / 2;
    let endind = startind.clone() + s1;
    (0..endind.len()).into_iter().for_each(|axis| {
        arr.slice_axis_inplace(
            Axis(axis),
            Slice::new(startind[axis] as isize, Some(endind[axis] as isize), 1),
        );
    });
}

/// Cross-correlate two N-dimensional arrays using FFT.
/// Complex conjugate of second array is calculate in function.
pub fn mfftcorrelate<A, S, D>(
    in1: &ArrayBase<S, D>,
    in2: &ArrayBase<S, D>,
    mode: Mode,
) -> Result<ArrayBase<OwnedRepr<A>, D>, Box<dyn Error>>
where
    A: FftNum + FromPrimitive + Default,
    S: Data<Elem = A>,
    D: Dimension,
{
    // check that arrays have the same number of dimensions
    if in1.ndim() != in2.ndim() {
        return Err("Input arrays must have the same number of dimensions.".into());
    }
    // reverse the second array
    let mut in2 = in2.to_owned();
    in2.slice_each_axis_inplace(|_| Slice::new(0, None, -1));

    let mut shape = in1.shape().to_vec();
    let s1 = Array::from_vec(
        in1.shape()
            .into_iter()
            .map(|a| *a as isize)
            .collect::<Vec<_>>(),
    );
    let s2 = Array::from_vec(
        in2.shape()
            .into_iter()
            .map(|a| *a as isize)
            .collect::<Vec<_>>(),
    );
    for (s, s_other) in shape.iter_mut().zip(in2.shape().iter()) {
        *s = *s + *s_other - 1;
    }
    let in1 = pad_with_zeros(in1, generate_pad_vector(&in1, shape.as_slice()))?;
    let in2 = pad_with_zeros(&in2, generate_pad_vector(&in2, shape.as_slice()))?;

    // multiple values in shape together to get total size
    let total_size = shape.iter().fold(1, |acc, x| acc * x);

    let mut in1 = in1.mapv(|x| Complex::new(x, Zero::zero()));
    let mut in2 = in2.mapv(|x| Complex::new(x, Zero::zero()));
    let t = Instant::now();
    in1.as_slice_mut().unwrap().fft_mut();
    in2.as_slice_mut().unwrap().fft_mut();
    println!("fft: {:?}", t.elapsed());

    // Multiply the FFTs.
    let t = Instant::now();
    let mut out = in1 * in2;
    println!("mul: {:?}", t.elapsed());

    // Perform the inverse FFT.
    let t = Instant::now();
    out.as_slice_mut().unwrap().ifft_mut();
    println!("inverse FFT: {:?}", t.elapsed());

    // Return the real part of the result. Note normalise by 1/total_size
    let total_size = A::from_usize(total_size).unwrap();
    // convert shape to a tuple of length shape.len()
    match mode {
        Mode::Full => {
            let out = out.mapv(|x| x.re / total_size);
            Ok(out)
        }
        Mode::Same => {
            let mut out = out.mapv(|x| x.re / total_size);
            centred(&mut out, s1);
            Ok(out)
        }
        Mode::Valid => {
            let mut out = out.mapv(|x| x.re / total_size);
            centred(&mut out, s1 - s2 + 1);
            Ok(out)
        }
    }
}

pub fn match_template(image: &Array2<f32>, kernel: &Array2<f32>) -> Array2<f32> {
    let start = Instant::now();
    let image = image.map(|&x| x as f64);
    let squared_image = image.map(|&x| x * x);
    let kernel = kernel.map(|&x| x as f64);
    println!("map to f64 cost: {}ms", start.elapsed().as_millis());
    let start = Instant::now();

    let mut res = mfftcorrelate(&image, &kernel, fftconvolve::Mode::Valid).unwrap();
    println!("fftcorrelate cost: {}ms", start.elapsed().as_millis());
    let start = Instant::now();

    let integral_image = integral_arr2(&image);
    let integral_squared_image = integral_arr2(&squared_image);
    println!(
        "integral and integral squared cost: {}ms",
        start.elapsed().as_millis()
    );
    let start = Instant::now();

    let kernel_sum = kernel.sum();
    let kernel_sqsum = kernel.map(|x| x * x).sum();

    let kernel_avg = kernel_sum / kernel.len() as f64;
    let kernel_var = kernel_sqsum / kernel.len() as f64 - kernel_avg * kernel_avg;
    println!("kernel avg and var cost: {}ms", start.elapsed().as_millis());
    let start = Instant::now();

    let (image_h, image_w) = image.dim();
    let (kernel_h, kernel_w) = kernel.dim();
    let (y_len, x_len) = (image_h - kernel_h + 1, image_w - kernel_w + 1);
    for x in 0..x_len {
        for y in 0..y_len {
            let value_sum = subsum_from_integral_arrf64(&integral_image, x, y, kernel_w, kernel_h);
            let value_sqsum =
                subsum_from_integral_arrf64(&integral_squared_image, x, y, kernel_w, kernel_h);

            let value_avg = value_sum / kernel.len() as f64;
            let value_var = value_sqsum / kernel.len() as f64 - value_avg * value_avg;

            let mut v = res[[y, x]];
            v -= value_sum * kernel_avg;

            let factor = (value_var * kernel_var).sqrt() * kernel.len() as f64;
            if v.abs() < factor {
                v /= factor;
            } else if v.abs() < 1.125 * factor {
                v = v.signum()
            } else {
                v = 0.0;
            }

            // if v.is_infinite() {
            //     println!("value_sum: {}, kernel_avg: {}, value_var: {}, kernel_var: {}", value_sum, kernel_avg, value_var, kernel_var);
            // }

            res.get_mut((y, x)).unwrap().assign_elem(v)
        }
    }
    println!("normalize cost: {}ms", start.elapsed().as_millis());

    // {
    //     let file = File::create("res.csv").unwrap();
    //     let mut writer = WriterBuilder::new().has_headers(false).from_writer(file);
    //     writer.serialize_array2(&res).unwrap();
    // }

    res.map(|&x| x as f32)
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

pub fn integral_arr2<T: AddAssign + SubAssign + Copy>(mat: &Array2<T>) -> Array2<T> {
    let (y_len, x_len) = mat.dim();

    let mut res = mat.clone();
    for cur_y in 0..y_len {
        for cur_x in 0..x_len {
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
    assert!(x + width - 1 < integral_mat.dim().1);
    assert!(y + height - 1 < integral_mat.dim().0);
    let left = x;
    let top = y;
    let right = x + width - 1;
    let bottom = y + height - 1;

    let mut res = integral_mat[[bottom, right]];
    // top left
    if let Some(&v) = integral_mat.get([top - 1, left - 1]) {
        res.add_assign(v);
    }
    // bottom left
    if let Some(&v) = integral_mat.get([bottom, left - 1]) {
        res.sub_assign(v);
    }
    // top right
    if let Some(&v) = integral_mat.get([top - 1, right]) {
        res.sub_assign(v);
    }
    res
}

pub fn subsum_from_integral_arrf64(
    integral_mat: &Array2<f64>,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> f64 {
    assert!(x + width - 1 < integral_mat.dim().1);
    assert!(y + height - 1 < integral_mat.dim().0);
    let left = x;
    let top = y;
    let right = x + width - 1;
    let bottom = y + height - 1;

    let res = integral_mat[[bottom, right]];
    if x > 0 && y > 0 {
        res + integral_mat[[top - 1, left - 1]]
            - integral_mat[[bottom, left - 1]]
            - integral_mat[[top - 1, right]]
    } else {
        if x > 0 {
            res - integral_mat[[bottom, left - 1]]
        } else if y > 0 {
            res - integral_mat[[top - 1, right]]
        } else {
            res
        }
    }
}

pub fn square_sum_arr2f32(mat: &Array2<f32>) -> f32 {
    mat.iter().map(|&p| p * p).sum()
}
