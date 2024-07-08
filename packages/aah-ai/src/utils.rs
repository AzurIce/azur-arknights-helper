use std::error::Error;

use image::DynamicImage;
use rten_tensor::{AsView, Tensor};

#[derive(Clone, Copy, PartialEq)]
pub enum ChannelOrder {
    Rgb,
    Bgr,
}

#[derive(Clone, Copy, PartialEq)]
pub enum DimOrder {
    /// Use "channels-first" order
    Nchw,
    /// Use "channels-last" order
    Nhwc,
}

/// Read an image from `path` into an NCHW or NHWC tensor, depending on
/// `out_dim_order`.
pub fn image_to_tensor/*<N: Fn(usize, f32) -> f32>*/(
    image: DynamicImage,
    // normalize_pixel: N,
    out_chan_order: ChannelOrder,
    out_dim_order: DimOrder,
    out_height: u32,
    out_width: u32,
) -> Result<Tensor<f32>, Box<dyn Error>> {
    let input_img = image.into_rgb8();

    // Resize the image using the `imageops::resize` function from the `image`
    // crate rather than using RTen's `resize` operator because
    // `imageops::resize` supports antialiasing. This significantly improves
    // output image quality and thus prediction accuracy when the output is
    // small (eg. 224 or 256px).
    //
    // The outputs of `imageops::resize` still don't match PyTorch exactly
    // though, which can lead to small differences in prediction outputs.
    let input_img = image::imageops::resize(
        &input_img,
        out_width,
        out_height,
        image::imageops::FilterType::Triangle,
    );

    let (width, height) = input_img.dimensions();

    // Map input channel index, in RGB order, to output channel index
    let out_chans = match out_chan_order {
        ChannelOrder::Rgb => [0, 1, 2],
        ChannelOrder::Bgr => [2, 1, 0],
    };

    let mut img_tensor = Tensor::zeros(&[1, 3, height as usize, width as usize]);
    for y in 0..height {
        for x in 0..width {
            for c in 0..3 {
                let pixel_value = input_img.get_pixel(x, y)[c] as f32 / 255.0;
                // let in_val = normalize_pixel(c, pixel_value);
                let in_val = pixel_value;
                img_tensor[[0, out_chans[c], y as usize, x as usize]] = in_val;
            }
        }
    }

    if out_dim_order == DimOrder::Nhwc {
        // NCHW => NHWC
        img_tensor.permute(&[0, 3, 2, 1]);
    }

    Ok(img_tensor)
}