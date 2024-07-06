use image::{ImageBuffer, Luma};
use wgpu::{util::DeviceExt, BindGroupDescriptor, ComputePipelineDescriptor, PipelineCompilationOptions};

use crate::{ccorr, gpu::Context};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderUniforms {
    input_width: u32,
    input_height: u32,
    template_width: u32,
    template_height: u32,
    tc_sq_sum: f32,
}

pub struct CcoeffNormedMatcher {
    ctx: Context,
    result_buffer: wgpu::Buffer,
    staging_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::ComputePipeline,

    res_w: u32,
    res_h: u32,
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use image::{ImageBuffer, Luma};

    use crate::utils::rgb_to_luma;

    use super::CcoeffNormedMatcher;

    #[test]
    fn test_ccoeff_normed_matcher() {
        let input = image::open("../../resources/templates/MUMU-1920x1080/start.png")
            .unwrap()
            .to_luma32f();
        let template = image::open("../../resources/templates/1920x1080/main_recruit.png")
            .unwrap()
            .to_luma32f();
        // let input = rgb_to_luma(&input);
        // let template = rgb_to_luma(&template);

        // let input = ImageBuffer::from_fn(7, 7, |x, y| Luma([x as f32 + y as f32]));
        // let template = ImageBuffer::from_fn(2, 2, |x, y| Luma([x as f32 + y as f32]));
        let matcher = CcoeffNormedMatcher::new(&input, &template, true);
        let res = matcher.run();
        let max = res
            .as_raw()
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap();
        let min = res
            .as_raw()
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap();
        let res = ImageBuffer::from_fn(res.width(), res.height(), |x, y| {
            Luma([((res.get_pixel(x, y).0[0] - min) / (max - min) * 255.0) as u8])
        });
        res.save("./output.png").unwrap();
    }
}

impl CcoeffNormedMatcher {
    pub fn new(
        input: &ImageBuffer<Luma<f32>, Vec<f32>>,
        template: &ImageBuffer<Luma<f32>, Vec<f32>>,
        padding: bool,
    ) -> Self {
        let ctx = pollster::block_on(Context::new());

        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("../../shaders/ccoeff_normed.wgsl"));

        let bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        // Input
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
                        // // Input Avg
                        // wgpu::BindGroupLayoutEntry {
                        //     binding: 1,
                        //     visibility: wgpu::ShaderStages::COMPUTE,
                        //     ty: wgpu::BindingType::Buffer {
                        //         ty: wgpu::BufferBindingType::Storage { read_only: true },
                        //         has_dynamic_offset: false,
                        //         min_binding_size: None,
                        //     },
                        //     count: None,
                        // },
                        // Template
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
                        // Result
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
                        // Uniform
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

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let input = if padding {
            let padded_w = input.width() + template.width() - 1;
            let padded_h = input.height() + template.height() - 1;

            ImageBuffer::from_fn(padded_w, padded_h, |x, y| {
                if x < input.width() && y < input.height() {
                    *input.get_pixel(x, y)
                } else {
                    Luma([0.0])
                }
            })
        } else {
            input.clone()
        };

        let avg_core = ImageBuffer::from_pixel(
            template.width(),
            template.height(),
            Luma([1.0 / (template.width() * template.height()) as f32]),
        );
        let input_avg = ccorr(&input, &avg_core, true);
        let input: ImageBuffer<Luma<f32>, Vec<f32>> = ImageBuffer::from_vec(
            input.width(),
            input.height(),
            input
                .as_raw()
                .iter()
                .zip(input_avg.data.iter())
                .map(|(a, b)| a - b)
                .collect(),
        )
        .unwrap();

        let template_avg = ccorr(template, &avg_core, true);
        let template: ImageBuffer<Luma<f32>, Vec<f32>> = ImageBuffer::from_vec(
            template.width(),
            template.height(),
            template
                .as_raw()
                .iter()
                .zip(template_avg.data.iter())
                .map(|(a, b)| a - b)
                .collect(),
        )
        .unwrap();

        let tc_sq_sum = template.pixels().map(|p| p[0] * p[0]).sum::<f32>();

        // let template_centered

        let input_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("input_buffer"),
                contents: bytemuck::cast_slice(&input.as_raw()),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let template_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("input_buffer"),
                contents: bytemuck::cast_slice(&template.as_raw()),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let uniforms = ShaderUniforms {
            input_width: input.width(),
            input_height: input.height(),
            template_width: template.width(),
            template_height: template.height(),
            tc_sq_sum,
        };
        let uniform_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("uniform_buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::bytes_of(&uniforms),
            });

        let res_w = input.width() - template.width() + 1;
        let res_h = input.height() - template.height() + 1;
        let res_buf_sz = (res_w * res_h) as u64 * size_of::<f32>() as u64;
        let result_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("result_buffer"),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            size: res_buf_sz,
            mapped_at_creation: false,
        });

        let bind_group = ctx.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding(),
                },
                // wgpu::BindGroupEntry {
                //     binding: 1,
                //     resource: input_avg_buffer.as_entire_binding(),
                // },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: template_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: result_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline = ctx
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "ccoeff_normed",
                compilation_options: PipelineCompilationOptions::default()
            });

        let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging_buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            size: res_buf_sz,
            mapped_at_creation: false,
        });

        Self {
            ctx,
            result_buffer,
            pipeline,
            staging_buffer,
            bind_group,
            res_w,
            res_h,
        }
    }

    /// Anchor on top left (0, 0)
    pub fn run(&self) -> ImageBuffer<Luma<f32>, Vec<f32>> {
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute_pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(
                (self.res_w as f32 / 16.0).ceil() as u32,
                (self.res_h as f32 / 16.0).ceil() as u32,
                1,
            );
        }

        let res_buf_sz = (self.res_w * self.res_h) as u64 * size_of::<f32>() as u64;
        encoder.copy_buffer_to_buffer(&self.result_buffer, 0, &self.staging_buffer, 0, res_buf_sz);

        self.ctx.queue.submit(std::iter::once(encoder.finish()));

        // get res
        let buffer_slice = self.staging_buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        self.ctx.device.poll(wgpu::Maintain::Wait);

        pollster::block_on(async {
            let result;

            if let Some(Ok(())) = receiver.receive().await {
                let data = buffer_slice.get_mapped_range();
                result = bytemuck::cast_slice(&data).to_vec();
                drop(data);
                self.staging_buffer.unmap();
            } else {
                result = vec![0.0; (self.res_w * self.res_h) as usize]
            };

            ImageBuffer::from_vec(self.res_w, self.res_h, result).unwrap()
        })
    }
}
