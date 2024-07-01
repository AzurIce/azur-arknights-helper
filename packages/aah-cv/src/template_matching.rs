use std::{
    iter::Sum,
    ops::{Add, AddAssign, Mul, Sub, SubAssign},
    time::Instant,
};

use imageproc::template_matching::Extremes;
use ndarray::{Array2, AssignElem};

use crate::convolve::gpu_convolve_block;



pub fn match_template(image: &Array2<f32>, kernel: &Array2<f32>) -> Array2<f32> {
    // let start = Instant::now();
    // let image = image.map(|&x| x as f64);
    let squared_image = image.map(|&x| x * x);
    // let kernel = kernel.map(|&x| x as f64);
    // println!("map to f64 cost: {}ms", start.elapsed().as_millis());

    let start = Instant::now();
    // let mut res = fftcorrelate(&image, &kernel, fftconvolve::Mode::Valid).unwrap();
    let mut res = gpu_convolve_block(&image, &kernel).unwrap();
    println!("correlate cost: {}ms", start.elapsed().as_millis());
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

    let kernel_avg = kernel_sum / kernel.len() as f32;
    let kernel_var = kernel_sqsum / kernel.len() as f32 - kernel_avg * kernel_avg;
    println!("kernel avg and var cost: {}ms", start.elapsed().as_millis());
    let start = Instant::now();

    let (image_h, image_w) = image.dim();
    let (kernel_h, kernel_w) = kernel.dim();
    let (y_len, x_len) = (image_h - kernel_h + 1, image_w - kernel_w + 1);
    for x in 0..x_len {
        for y in 0..y_len {
            let value_sum = subsum_from_integral(&integral_image, x, y, kernel_w, kernel_h);
            let value_sqsum =
                subsum_from_integral(&integral_squared_image, x, y, kernel_w, kernel_h);

            let value_avg = value_sum / kernel.len() as f32;
            let value_var = value_sqsum / kernel.len() as f32 - value_avg * value_avg;

            let mut v = res[[y, x]];
            v -= value_sum * kernel_avg;

            let factor = (value_var * kernel_var).sqrt() * kernel.len() as f32;
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

/// Calculate the prefix sum of the mat
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

/// Calculate the sum of the sub-matrix from (x, y) with width and height through the integral matrix
pub fn subsum_from_integral<T: Add<T, Output = T> + Sub<T, Output = T> + Copy>(
    integral_mat: &Array2<T>,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> T {
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

/// Calculate the sum of the square of the elements in the matrix
pub fn square_sum_arr2<T: Mul<T, Output = T> + Sum + Copy>(mat: &Array2<T>) -> T {
    mat.iter().map(|&p| p * p).sum()
}

#[cfg(test)]
mod test {
    use ndarray::{Array2, ArrayBase, OwnedRepr};
    use super::*;

    #[test]
    fn test_integral() {
        let mat = Array2::ones((5, 5));
        let integral: ArrayBase<OwnedRepr<_>, ndarray::prelude::Dim<[usize; 2]>> =
            integral_arr2(&mat);
        println!("{:?}", integral);
        assert_eq!(
            integral,
            Array2::from_shape_fn((5, 5), |(y, x)| { (x as f32 + 1.0) * (y as f32 + 1.0) })
        );
        let res = subsum_from_integral::<f32>(&integral, 2, 2, 3, 3);
        assert_eq!(res, 9.0);
        let res = subsum_from_integral(&integral, 0, 2, 2, 2);
        assert_eq!(res, 4.0);
        let res = subsum_from_integral(&integral, 0, 0, 2, 2);
        assert_eq!(res, 4.0);
    }
}