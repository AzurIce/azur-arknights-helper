use std::borrow::Cow;

use ndarray::Zip;
use ndarray::{Array2, ArrayView2, ArrayViewMut2, Axis};
use wgpu::util::DeviceExt;

fn convolve(image: &Array2<f32>, kernel: &Array2<f32>) -> Array2<f32> {
    let (image_height, image_width) = image.dim();
    let (kernel_height, kernel_width) = kernel.dim();
    let pad_height = kernel_height / 2;
    let pad_width = kernel_width / 2;

    let result_height = image_height - kernel_height + 1;
    let result_width = image_width - kernel_width + 1;

    let mut result = Array2::zeros((result_height, result_width));

    for ((y, x), res) in result.indexed_iter_mut() {
        let y_end = y + kernel_height;
        let x_end = x + kernel_width;

        let mut sum = 0.0;
        for ky in y..y_end {
            for kx in x..x_end {
                sum += image[[ky, kx]] * kernel[[ky - y, kx - x]];
            }
        }
        *res = sum;
    }

    result
}

// gpu

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderUniforms {
    image_width: u32,
    image_height: u32,
    kernel_width: u32,
    kernel_height: u32,
}

fn execute_gpu_block(image: &Array2<f32>, kernel: &Array2<f32>) -> Option<Array2<f32>> {
    pollster::block_on(execute_gpu(image, kernel))
}

async fn execute_gpu(image: &Array2<f32>, kernel: &Array2<f32>) -> Option<Array2<f32>> {
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
            },
            None,
        )
        .await
        .unwrap();

    execute_gpu_inner(&device, &queue, image, kernel).await
}

async fn execute_gpu_inner(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: &Array2<f32>,
    kernel: &Array2<f32>,
) -> Option<Array2<f32>> {
    // let mut input = input.iter().cloned().collect::<Vec<[f32; 2]>>();
    // bit_reverse_swap(&mut input);

    // let n = input.len() as u32;
    // println!("input len {n}: {:?}", input);

    // Loads the shader from WGSL
    let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    // Gets the size in bytes of the buffer.
    // let size = std::mem::size_of_val(&input) as wgpu::BufferAddress;
    // dimension 0 is height, dimension 1 is width
    let image_height = image.shape()[0];
    let image_width = image.shape()[1];
    let kernel_height = kernel.shape()[0];
    let kernel_width = kernel.shape()[1];

    let result_width = image_width - kernel_width + 1;
    let result_height = image_height - kernel_height + 1;

    let result_size_in_byte = (result_width * result_height * std::mem::size_of::<f32>()) as u64;

    let uniform = ShaderUniforms {
        image_width: image_width as u32,
        image_height: image_height as u32,
        kernel_width: kernel_width as u32,
        kernel_height: kernel_height as u32,
    };
    println!("{:?}", uniform);
    println!("result size: {}x{}", result_width, result_height);

    // Instantiates buffer without data.
    // `usage` of buffer specifies how it can be used:
    //   `BufferUsages::MAP_READ` allows it to be read (outside the shader).
    //   `BufferUsages::COPY_DST` allows it to be the destination of the copy.
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: result_size_in_byte,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let result_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: result_size_in_byte,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // Instantiates buffer with data (`numbers`).
    // Usage allowing the buffer to be:
    //   A storage buffer (can be bound within a bind group and thus available to a shader).
    //   The destination of a copy.
    //   The source of a copy.
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::bytes_of(&uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let image_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Image Buffer"),
        contents: bytemuck::cast_slice(&image.as_slice().unwrap()),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });
    let kernel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Kernel Buffer"),
        contents: bytemuck::cast_slice(&kernel.as_slice().unwrap()),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    // A bind group defines how buffers are accessed by shaders.
    // It is to WebGPU what a descriptor set is to Vulkan.
    // `binding` here refers to the `binding` of a buffer in the shader (`layout(set = 0, binding = 0) buffer`).

    // A pipeline specifies the operation of a shader

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bindgroup layout"),
        entries: &[
            // Uniform
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Image
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
            // Kernel
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Result
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
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
        // compilation_options: Default::default(),
    });

    // Instantiates the bind group, once again specifying the binding of buffers.
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: image_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: kernel_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: result_buffer.as_entire_binding(),
            },
        ],
    });

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
        cpass.dispatch_workgroups(
            (result_width as f32 / 16.0).ceil() as u32,
            (result_height as f32 / 16.0).ceil() as u32,
            1,
        ); // Number of cells to run, the (x,y,z) size of item being processed
    }
    queue.submit(Some(encoder.finish()));
    // ? get result
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // Sets adds copy operation to command encoder.
    // Will copy data from storage buffer on GPU to staging buffer on CPU.
    encoder.copy_buffer_to_buffer(&result_buffer, 0, &staging_buffer, 0, result_size_in_byte);
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
    if let Ok(Ok(())) = receiver.recv_async().await {
        // Gets contents of buffer
        let data = buffer_slice.get_mapped_range();
        // Since contents are got in bytes, this converts these bytes back to u32
        let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        staging_buffer.unmap(); // Unmaps buffer from memory
                                // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                //   delete myPointer;
                                //   myPointer = NULL;
                                // It effectively frees the memory

        // Returns data from buffer
        let result = Array2::from_shape_vec((result_height, result_width), result).unwrap();
        Some(result)
    } else {
        panic!("failed to run compute on gpu!")
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use ndarray::Array2;

    use crate::convolve::{convolve, execute_gpu_block};

    fn test_convolve_with_size(image_size: usize, kernel_size: usize) {
        println!("testing in image_size {image_size} and kernel_size {kernel_size}...");
        #[rustfmt::skip]
		let image = Array2::from_shape_fn((image_size, image_size), |(x, y)| {
			(x + y * image_size) as f32
		});

        #[rustfmt::skip]
		let kernel = Array2::from_shape_fn((kernel_size, kernel_size), |(x, y)| {
			(x + y * kernel_size) as f32
		});

        let t = Instant::now();
        let result = convolve(&image, &kernel);
        println!("naive cost: {:?}", t.elapsed());
        // println!("{:?}", result);

        let t = Instant::now();
        let result = execute_gpu_block(&image, &kernel).unwrap();
        println!("gpu cost: {:?}", t.elapsed());
        // println!("{:?}", result);
    }

    #[test]
    fn test_main() {
		/*
		testing in image_size 10 and kernel_size 3...
		naive cost: 76µs
		ShaderUniforms { image_width: 10, image_height: 10, kernel_width: 3, kernel_height: 3 }
		result size: 8x8
		gpu cost: 50.728334ms
		testing in image_size 128 and kernel_size 3...
		naive cost: 10.764625ms
		ShaderUniforms { image_width: 128, image_height: 128, kernel_width: 3, kernel_height: 3 }
		result size: 126x126
		gpu cost: 5.632792ms
		testing in image_size 1024 and kernel_size 3...
		naive cost: 706.569125ms
		ShaderUniforms { image_width: 1024, image_height: 1024, kernel_width: 3, kernel_height: 3 }
		result size: 1022x1022
		gpu cost: 8.917958ms
		testing in image_size 2048 and kernel_size 32...
		naive cost: 231.010488917s
		ShaderUniforms { image_width: 2048, image_height: 2048, kernel_width: 32, kernel_height: 32 }
		result size: 2017x2017
		gpu cost: 79.495083ms
		 */
		test_convolve_with_size(10, 3);
		test_convolve_with_size(128, 3);
		test_convolve_with_size(1024, 3);
		test_convolve_with_size(2048, 32);
	}
}
