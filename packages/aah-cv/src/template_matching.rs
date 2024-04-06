use std::{
    ops::{AddAssign, SubAssign},
    time::Instant,
};

use fftconvolve::fftcorrelate;
use imageproc::template_matching::Extremes;
use ndarray::{Array2, AssignElem, Dim};
use wgpu::util::DeviceExt;

#[cfg(test)]
mod test {
    use std::{path::Path, time::Instant};

    use fft2d::slice::fft_2d;
    use image::GrayImage;
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

    fn test_template_match_with_image_and_template(image: &str, template: &str) {
        println!("matching {} {}...", image, template);
        let image = image::open(Path::new("./test").join(image)).unwrap();
        let template = image::open(Path::new("./test").join(template)).unwrap();

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
        /*
        matching start.png start_btn.png...
        map to f64 cost: 68ms
        fftcorrelate cost: 17940ms
        integral and integral squared cost: 1644ms
        kernel avg and var cost: 0ms
        normalize cost: 1932ms
        aah-cv: Extremes { max_value: 0.9999873, min_value: -0.38987702, max_value_location: (1253, 1336), min_value_location: (541, 71) }, cost: 21.77749s
        test template_matching::test::test_template_match ... ok
         */
        // test_template_match_with_image_and_template("start.png", "start_btn.png");
        /*
        matching main.png EnterMissionMistCity.png...
        map to f64 cost: 67ms
        fftcorrelate cost: 23164ms
        integral and integral squared cost: 1533ms
        kernel avg and var cost: 0ms
        normalize cost: 1588ms
        aah-cv: Extremes { max_value: 0.99995416, min_value: -0.42217892, max_value_location: (1445, 1105), min_value_location: (1679, 945) }, cost: 26.527304s
        test template_matching::test::test_template_match ... ok
         */
        // test_template_match_with_image_and_template("main.png", "EnterMissionMistCity.png");
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

pub fn match_template(image: &Array2<f32>, kernel: &Array2<f32>) -> Array2<f32> {
    let start = Instant::now();
    let image = image.map(|&x| x as f64);
    let squared_image = image.map(|&x| x * x);
    let kernel = kernel.map(|&x| x as f64);
    println!("map to f64 cost: {}ms", start.elapsed().as_millis());
    let start = Instant::now();

    let mut res = fftcorrelate(&image, &kernel, fftconvolve::Mode::Valid).unwrap();
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderUniforms {
    input_width: u32,
    input_height: u32,
    template_width: u32,
    template_height: u32,
}

pub struct TemplateMatcher {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    shader: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,

    last_pipeline: Option<wgpu::ComputePipeline>,
    // last_method: Option<MatchTemplateMethod>,

    last_input_size: (u32, u32),
    last_template_size: (u32, u32),
    last_result_size: (u32, u32),

    uniform_buffer: wgpu::Buffer,
    input_buffer: Option<wgpu::Buffer>,
    template_buffer: Option<wgpu::Buffer>,
    result_buffer: Option<wgpu::Buffer>,
    staging_buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,

    matching_ongoing: bool,
}

impl Default for TemplateMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateMatcher {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = pollster::block_on(async {
            instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await
                .expect("Adapter request failed")
        });

        let (device, queue) = pollster::block_on(async {
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await
                .expect("Device request failed")
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/matching.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform_buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: std::mem::size_of::<ShaderUniforms>() as _,
            mapped_at_creation: false,
        });

        Self {
            instance,
            adapter,
            device,
            queue,
            shader,
            pipeline_layout,
            bind_group_layout,
            last_pipeline: None,
            // last_method: None,
            last_input_size: (0, 0),
            last_template_size: (0, 0),
            last_result_size: (0, 0),
            uniform_buffer,
            input_buffer: None,
            template_buffer: None,
            result_buffer: None,
            staging_buffer: None,
            bind_group: None,
            matching_ongoing: false,
        }
    }

    /// Waits for the latest [match_template] execution and returns the result.
    /// Returns [None] if no matching was started.
    pub fn wait_for_result(&mut self) -> Option<Array2<f32>> {
        if !self.matching_ongoing {
            return None;
        }
        self.matching_ongoing = false;

        let (result_width, result_height) = self.last_result_size;

        let buffer_slice = self.staging_buffer.as_ref().unwrap().slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        self.device.poll(wgpu::Maintain::Wait);

        pollster::block_on(async {
            let result;

            if let Some(Ok(())) = receiver.receive().await {
                let data = buffer_slice.get_mapped_range();
                result = bytemuck::cast_slice(&data).to_vec();
                drop(data);
                self.staging_buffer.as_ref().unwrap().unmap();
            } else {
                result = vec![0.0; (result_width * result_height) as usize]
            };

            Array2::from_shape_vec((result_width as usize, result_height as usize), result).ok()
        })
    }

    /// Slides a template over the input and scores the match at each point using the requested method.
    /// To get the result of the matching, call [wait_for_result].
    pub fn match_template<'a>(
        &mut self,
        input: Array2<f32>,
        template: Array2<f32>,
        // method: MatchTemplateMethod,
    ) {
        if self.matching_ongoing {
            // Discard previous result if not collected.
            self.wait_for_result();
        }

        // let input = input.into();
        // let template = template.into();

        if self.last_pipeline.is_none() /* || self.last_method != Some(method) */ {
            // self.last_method = Some(method);

            // let entry_point = match method {
            //     MatchTemplateMethod::SumOfAbsoluteErrors => "main_sae",
            //     MatchTemplateMethod::SumOfSquaredErrors => "main_sse",
            //     MatchTemplateMethod::CrossCorrelation => "main_cc",
            // };
            let entry_point = "main";

            self.last_pipeline = Some(self.device.create_compute_pipeline(
                &wgpu::ComputePipelineDescriptor {
                    label: None,
                    layout: Some(&self.pipeline_layout),
                    module: &self.shader,
                    entry_point,
                },
            ));
        }

        let mut buffers_changed = false;

        let input_size = (input.dim().0 as u32, input.dim().1 as u32);
        if self.input_buffer.is_none() || self.last_input_size != input_size {
            buffers_changed = true;

            self.last_input_size = input_size;

            self.input_buffer = Some(self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("input_buffer"),
                    contents: bytemuck::cast_slice(&input.as_slice().unwrap()),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                },
            ));
        } else {
            self.queue.write_buffer(
                self.input_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&input.as_slice().unwrap()),
            );
        }

        let template_size = (template.dim().0 as u32, template.dim().1 as u32);
        if self.template_buffer.is_none() || self.last_template_size != template_size {
            self.queue.write_buffer(
                &self.uniform_buffer,
                0,
                bytemuck::cast_slice(&[ShaderUniforms {
                    input_width: input_size.0,
                    input_height: input_size.1,
                    template_width: template_size.0,
                    template_height: template_size.1,
                }]),
            );
            buffers_changed = true;

            self.last_template_size = template_size;

            self.template_buffer = Some(self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("template_buffer"),
                    contents: bytemuck::cast_slice(&template.as_slice().unwrap()),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                },
            ));
        } else {
            self.queue.write_buffer(
                self.template_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&template.as_slice().unwrap()),
            );
        }

        let result_width = input_size.0 - template_size.0 + 1;
        let result_height = input_size.1 - template_size.1 + 1;
        let result_buf_size = (result_width * result_height) as u64 * std::mem::size_of::<f32>() as u64;

        if buffers_changed {
            self.last_result_size = (result_width, result_height);

            self.result_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("result_buffer"),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                size: result_buf_size,
                mapped_at_creation: false,
            }));

            self.staging_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("staging_buffer"),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                size: result_buf_size,
                mapped_at_creation: false,
            }));

            self.bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.input_buffer.as_ref().unwrap().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.template_buffer.as_ref().unwrap().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.result_buffer.as_ref().unwrap().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                ],
            }));
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute_pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(self.last_pipeline.as_ref().unwrap());
            compute_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            compute_pass.dispatch_workgroups(
                (result_width as f32 / 16.0).ceil() as u32,
                (result_height as f32 / 16.0).ceil() as u32,
                1,
            );
        }

        encoder.copy_buffer_to_buffer(
            self.result_buffer.as_ref().unwrap(),
            0,
            self.staging_buffer.as_ref().unwrap(),
            0,
            result_buf_size,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        self.matching_ongoing = true;
    }
}