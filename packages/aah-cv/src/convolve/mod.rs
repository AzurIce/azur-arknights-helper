use std::borrow::Cow;
use std::time::Instant;

use imageproc::filter::Kernel;
use ndarray::Zip;
use ndarray::{Array2, ArrayView2, ArrayViewMut2, Axis};
use wgpu::util::DeviceExt;

use crate::gpu::{
    BindGroupEntriesBuilder, BindGroupLayoutEntriesBuilder, Context, GpuTask, GpuTaskWrapper,
};

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

pub fn gpu_convolve_block(image: &Array2<f32>, kernel: &Array2<f32>) -> Option<Array2<f32>> {
    pollster::block_on(gpu_convolve(image, kernel))
}

pub async fn gpu_convolve(image: &Array2<f32>, kernel: &Array2<f32>) -> Option<Array2<f32>> {
    let image_height = image.shape()[0];
    let image_width = image.shape()[1];
    let kernel_height = kernel.shape()[0];
    let kernel_width = kernel.shape()[1];

    let result_width = image_width - kernel_width + 1;
    let result_height = image_height - kernel_height + 1;

    let result_size_in_byte = (result_width * result_height * std::mem::size_of::<f32>()) as u64;

    let t = Instant::now();
    let context = Context::new().await;
    println!("context: {:?}", t.elapsed());
    let task = GpuConvolveTask::new(&context, image, kernel).await;

    let task = GpuTaskWrapper::<f32>::new(context, result_size_in_byte, Box::new(task)).await;

    let res = task.exec().await;
    res.map(|res| Array2::from_shape_vec((result_height, result_width), res).unwrap())
}

pub struct GpuConvolveTask {
    image: Array2<f32>,
    kernel: Array2<f32>,
    uniform_buffer: wgpu::Buffer,
    image_buffer: wgpu::Buffer,
    kernel_buffer: wgpu::Buffer,
}

impl GpuConvolveTask {
    pub async fn new(context: &Context, image: &Array2<f32>, kernel: &Array2<f32>) -> Self {
        // Gets the size in bytes of the buffer.
        // let size = std::mem::size_of_val(&input) as wgpu::BufferAddress;
        // dimension 0 is height, dimension 1 is width
        let image_height = image.shape()[0];
        let image_width = image.shape()[1];
        let kernel_height = kernel.shape()[0];
        let kernel_width = kernel.shape()[1];

        let result_width = image_width - kernel_width + 1;
        let result_height = image_height - kernel_height + 1;

        let uniform = ShaderUniforms {
            image_width: image_width as u32,
            image_height: image_height as u32,
            kernel_width: kernel_width as u32,
            kernel_height: kernel_height as u32,
        };
        println!("{:?}", uniform);
        println!("result size: {}x{}", result_width, result_height);

        // Instantiates buffer with data (`numbers`).
        // Usage allowing the buffer to be:
        //   A storage buffer (can be bound within a bind group and thus available to a shader).
        //   The destination of a copy.
        //   The source of a copy.
        let uniform_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::bytes_of(&uniform),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let image_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Image Buffer"),
                contents: bytemuck::cast_slice(&image.as_slice().unwrap()),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            });
        let kernel_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Kernel Buffer"),
                contents: bytemuck::cast_slice(&kernel.as_slice().unwrap()),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            });

        Self {
            image: image.clone(),
            kernel: kernel.clone(),
            uniform_buffer,
            image_buffer,
            kernel_buffer,
        }
    }
}

impl GpuTask for GpuConvolveTask {
    fn build_bind_group_entries<'a>(
        &'a self,
        bind_group_entry_builder: BindGroupEntriesBuilder<'a>,
    ) -> BindGroupEntriesBuilder {
        bind_group_entry_builder
            .add_buffer(&self.uniform_buffer)
            .add_buffer(&self.image_buffer)
            .add_buffer(&self.kernel_buffer)
    }

    fn build_bind_group_layout_entries(
        &self,
        bind_group_layout_entry_builder: BindGroupLayoutEntriesBuilder,
    ) -> BindGroupLayoutEntriesBuilder {
        bind_group_layout_entry_builder
            .add_buffer(wgpu::BufferBindingType::Uniform)
            .add_buffer(wgpu::BufferBindingType::Storage { read_only: true })
            .add_buffer(wgpu::BufferBindingType::Storage { read_only: true })
    }

    fn cs_module(&self, device: &wgpu::Device) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        })
    }

    fn exec(&self, cpass: &mut wgpu::ComputePass) {
        let image_height = self.image.shape()[0];
        let image_width = self.image.shape()[1];
        let kernel_height = self.kernel.shape()[0];
        let kernel_width = self.kernel.shape()[1];

        let result_width = image_width - kernel_width + 1;
        let result_height = image_height - kernel_height + 1;
        cpass.dispatch_workgroups(
            (result_width as f32 / 16.0).ceil() as u32,
            (result_height as f32 / 16.0).ceil() as u32,
            1,
        ); // Number of cells to run, the (x,y,z) size of item being processed
    }
}
#[cfg(test)]
mod test {
    use std::time::Instant;

    use ndarray::Array2;

    use crate::convolve::{convolve, gpu_convolve, gpu_convolve_block};

    fn test_convolve_with_size(image_size: usize, kernel_size: usize) {
        println!("testing in image_size {image_size} and kernel_size {kernel_size}...");
        #[rustfmt::skip]
		let image = Array2::from_shape_fn((image_size, image_size), |(x, y)| {
			(x + y * image_size) as f32
		});

        #[rustfmt::skip]
		let kernel = Array2::from_shape_fn((kernel_size, kernel_size), |(x, y)| {
			(x + y * kernel_size) as f32
            // y as f32
            // x as f32
		});

        let t = Instant::now();
        let result = convolve(&image, &kernel);
        println!("naive cost: {:?}", t.elapsed());
        println!("{:?}", result);

        let t = Instant::now();
        let result = pollster::block_on(gpu_convolve(&image, &kernel)).unwrap();
        // let result = gpu_convolve_block(&image, &kernel).unwrap();
        println!("gpu cost: {:?}", t.elapsed());
        println!("{:?}", result);
    }

    #[test]
    fn test_main() {
        /*
        context: 40ms

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
        // test_convolve_with_size(10, 3);
        // test_convolve_with_size(128, 3);
        test_convolve_with_size(1024, 3);
        // test_convolve_with_size(2048, 32);
    }
}
