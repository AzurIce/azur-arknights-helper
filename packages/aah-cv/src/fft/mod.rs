use nalgebra::Complex;
use std::{borrow::Cow, str::FromStr};
use wgpu::{util::DeviceExt, BufferBinding};

pub fn bit_reverse_swap<T>(input: &mut [T]) {
    // do bit reverse swap on input
    let n = input.len();

    let mut j = 0;
    for i in 0..n {
        if i < j {
            input.swap(i, j);
        }

        let mut l = n / 2;
        loop {
            j ^= l;
            if j >= l {
                break;
            }
            l >>= 1;
        }
    }
}

fn inner_fft(buf: &mut [Complex<f64>], omega: &[Complex<f64>]) {
    bit_reverse_swap(buf);

    let n = buf.len();

    let mut l = 2;
    while l <= n {
        let m = l / 2;
        for p in 0..n / l {
            for i in 0..m {
                let t = omega[n / l * i] * buf[p * l + m + i];
                buf[p * l + m + i] = buf[p * l + i] - t;
                buf[p * l + i] += t;
            }
        }
        l <<= 1;
    }
}

/// input has to be 2^n
pub fn fft(input: Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    let mut output = input;

    let n = output.len();

    let omega = (0..n)
        .map(|i| {
            let theta = 2.0 * std::f64::consts::PI / n as f64 * i as f64;
            Complex::new(theta.cos(), theta.sin())
        })
        .collect::<Vec<_>>();
    // println!("!!!omega!!!: {:?}", omega);

    inner_fft(&mut output, &omega);

    output
}

pub fn ifft(input: Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    let mut output = input;

    let n = output.len();

    let omega = (0..n)
        .map(|i| {
            let theta = 2.0 * std::f64::consts::PI / n as f64 * i as f64;
            Complex::new(theta.cos(), theta.sin())
        })
        .collect::<Vec<_>>();
    let omega_inverse = omega.iter().map(|&x| x.conj()).collect::<Vec<_>>();

    inner_fft(&mut output, &omega_inverse);

    output.iter().map(|&x| x / n as f64).collect()
}

// Indicates a u32 overflow in an intermediate Collatz value
const OVERFLOW: u32 = 0xffffffff;

// #[cfg_attr(test, allow(dead_code))]
// async fn run() {
//     let numbers = (0..8).map(|i| [i as f32, 0.0]).collect::<Vec<[f32; 2]>>();

//     let steps = execute_gpu(&numbers).await.unwrap();

//     let disp_steps: Vec<String> = steps
//         .iter()
//         .map(|&n| match n {
//             OVERFLOW => "OVERFLOW".to_string(),
//             _ => n.to_string(),
//         })
//         .collect();

//     println!("Steps: [{}]", disp_steps.join(", "));
//     #[cfg(target_arch = "wasm32")]
//     log::info!("Steps: [{}]", disp_steps.join(", "));
// }

fn execute_gpu_block(numbers: &[[f32; 2]]) -> Option<Vec<[f32; 2]>> {
    pollster::block_on(execute_gpu(numbers))
}

async fn execute_gpu(numbers: &[[f32; 2]]) -> Option<Vec<[f32; 2]>> {
    // Instantiates instance of WebGPU
    let instance = wgpu::Instance::default();

    // `request_adapter` instantiates the general connection to the GPU
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await?;

    // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
    //  `features` being the available features.
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();

    execute_gpu_inner(&device, &queue, numbers).await
}

async fn execute_gpu_inner(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    input: &[[f32; 2]],
) -> Option<Vec<[f32; 2]>> {
    let mut input = input.iter().cloned().collect::<Vec<[f32; 2]>>();
    bit_reverse_swap(&mut input);

    let n = input.len() as u32;
    // println!("input len {n}: {:?}", input);

    // Loads the shader from WGSL
    let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    // Gets the size in bytes of the buffer.
    // let size = std::mem::size_of_val(&input) as wgpu::BufferAddress;
    let size = input.len() as u64 * std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress;

    // Instantiates buffer without data.
    // `usage` of buffer specifies how it can be used:
    //   `BufferUsages::MAP_READ` allows it to be read (outside the shader).
    //   `BufferUsages::COPY_DST` allows it to be the destination of the copy.
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uniform_n_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("n"),
        size: 4,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let uniform_l_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("l"),
        size: 4,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Instantiates buffer with data (`numbers`).
    // Usage allowing the buffer to be:
    //   A storage buffer (can be bound within a bind group and thus available to a shader).
    //   The destination of a copy.
    //   The source of a copy.
    let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::cast_slice(&input),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let omega = (0..n)
        .map(|i| {
            let theta = 2.0 * std::f64::consts::PI / n as f64 * i as f64;
            [theta.cos() as f32, theta.sin() as f32]

            // Complex::new(theta.cos(), theta.sin())
        })
        .collect::<Vec<_>>();
    // let omega_inverse = omega.iter().map(|&[r, i]| [r, -i]).collect::<Vec<_>>();

    let omega_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Omega Buffer"),
        contents: bytemuck::cast_slice(&omega),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    // A bind group defines how buffers are accessed by shaders.
    // It is to WebGPU what a descriptor set is to Vulkan.
    // `binding` here refers to the `binding` of a buffer in the shader (`layout(set = 0, binding = 0) buffer`).

    // A pipeline specifies the operation of a shader

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bindgroup layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
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
        label: Some("Pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        ..Default::default()
    });

    // Instantiates the pipeline.
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &cs_module,
        entry_point: "main",
    });

    // Instantiates the bind group, once again specifying the binding of buffers.
    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: omega_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniform_n_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: uniform_l_buffer.as_entire_binding(),
            },
        ],
    });

    queue.write_buffer(&uniform_n_buffer, 0, bytemuck::bytes_of(&n));

    let mut l = 2;
    let mut res: Option<Vec<[f32; 2]>> = None;
    while l <= n {
        // println!("iter {l}");
        // let m = l / 2;

        // iter(l as u32);
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // let mut iter = |l: u32| {
        queue.write_buffer(&uniform_l_buffer, 0, bytemuck::bytes_of(&l));
        // A command encoder executes one or many pipelines.
        // It is to WebGPU what a command buffer is to Vulkan.
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_pipeline(&compute_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.insert_debug_marker("compute collatz iterations");
            cpass.dispatch_workgroups(n as u32 / l as u32, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
        }
        queue.submit(Some(encoder.finish()));
        // };
        // for p in 0..n/l {
        // for i in 0..m {
        // let t = omega[n / l * i] * buf[p*l + m + i];
        // buf[p*l + m + i] = buf[p*l + i] - t;
        // buf[p*l + i] += t;
        // }
        // }
        // println!("{:?}", res);

        l <<= 1;
    }
    // ? get result
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    // Sets adds copy operation to command encoder.
    // Will copy data from storage buffer on GPU to staging buffer on CPU.
    encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, size);
    // encoder.copy_buffer_to_buffer(&omega_buffer, 0, &staging_buffer, 0, size);

    // Submits command encoder for processing
    queue.submit(Some(encoder.finish()));

    // Note that we're not calling `.await` here.
    let buffer_slice = staging_buffer.slice(..);
    // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
    device.poll(wgpu::Maintain::wait()).panic_on_timeout();

    // Awaits until `buffer_future` can be read from
    res = if let Ok(Ok(())) = receiver.recv_async().await {
        // Gets contents of buffer
        let data = buffer_slice.get_mapped_range();
        // Since contents are got in bytes, this converts these bytes back to u32
        let result: Vec<[f32; 2]> = bytemuck::cast_slice(&data).to_vec();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        staging_buffer.unmap(); // Unmaps buffer from memory
                                // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                //   delete myPointer;
                                //   myPointer = NULL;
                                // It effectively frees the memory

        // Returns data from buffer
        Some(result)
    } else {
        panic!("failed to run compute on gpu!")
    };
    res
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use num::Complex;
    use rustfft::FftPlanner;

    use crate::fft::{bit_reverse_swap, ifft};

    use super::{execute_gpu_block, fft};

    #[test]
    pub fn test_bit_reverse_swap() {
        let mut data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        bit_reverse_swap(&mut data);
        assert_eq!(data, vec![1, 5, 3, 7, 2, 6, 4, 8]);
    }

    #[test]
    pub fn test_fft() {
        let data = (0..8)
            .map(|i| Complex::new(i as f64, 0.0))
            .collect::<Vec<Complex<f64>>>();
        // println!("original: {:?}", data);

        let t = Instant::now();
        let data = fft(data);
        println!("fft cost: {:?}", t.elapsed()); // 1.1ms
        println!("{:?}", data);

        let t = Instant::now();
        let data = ifft(data);
        println!("ifft cost: {:?}", t.elapsed()); // 1.2ms
        println!("{:?}", data);

        println!("-------------------------");

        // let mut data = (0..8)
        //     .map(|i| Complex::new(i as f64, 0.0))
        //     .collect::<Vec<Complex<f64>>>();
        // let mut planner = FftPlanner::new();
        // let fft = planner.plan_fft_forward(8);
        // let inv_fft = planner.plan_fft_inverse(8);

        // fft.process(&mut data);
        // println!("fft: {:?}", data);

        // inv_fft.process(&mut data);
        // println!("inv_fft: {:?}", data)
    }

    fn test_gpu_fft_with_size(size: usize) {
        println!("testing in size {size}...");
        let data = (0..size)
            .map(|i| Complex::new(i as f64, 0.0))
            .collect::<Vec<Complex<f64>>>();
        // println!("original: {:?}", data);

        let t = Instant::now();
        let data = fft(data);
        println!("naive fft cost: {:?}", t.elapsed()); // 1.1ms
                                                       // println!("{:?}", data);

        let t = Instant::now();
        let numbers = (0..size)
            .map(|i| [i as f32, 0.0])
            .collect::<Vec<[f32; 2]>>();
        let res = execute_gpu_block(&numbers);
        println!("gpu fft cost: {:?}", t.elapsed()); // 1.1ms
                                                     // println!("{:?}", res)
    }

    #[test]
    pub fn test_gpu_fft() {
        test_gpu_fft_with_size(65536);
        // test_gpu_fft_with_size(1280 * 720);
    }
}

// pub fn main() {
//     #[cfg(not(target_arch = "wasm32"))]
//     {
//         env_logger::init();
//         pollster::block_on(run());
//     }
//     #[cfg(target_arch = "wasm32")]
//     {
//         std::panic::set_hook(Box::new(console_error_panic_hook::hook));
//         console_log::init().expect("could not initialize logger");
//         wasm_bindgen_futures::spawn_local(run());
//     }
// }
