//! A cross correlation matcher

use bytemuck::{Pod, Zeroable};
use image::{ImageBuffer, Luma};
use wgpu::{
    include_wgsl, util::DeviceExt, BindGroup, BindGroupDescriptor, BindGroupLayoutDescriptor, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, PipelineCompilationOptions, PipelineLayoutDescriptor
};

use crate::gpu::Context;

pub struct CrossCorrelationMatcher {
    ctx: Context,

    input_buffer: Option<wgpu::Buffer>,
    template_buffer: Option<wgpu::Buffer>,
    result_buffer: Option<wgpu::Buffer>,
    staging_buffer: Option<wgpu::Buffer>,
    uniform_buffer: wgpu::Buffer,

    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    bind_group: Option<wgpu::BindGroup>,
    pipeline: wgpu::ComputePipeline,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Uniforms {
    image_width: u32,
    image_height: u32,
    template_width: u32,
    template_height: u32,
}

impl CrossCorrelationMatcher {
    pub fn new() -> Self {
        let ctx = pollster::block_on(Context::new());

        let bind_group_layout = ctx
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("CrossCorrelationMatcher BindGroupLayout"),
                entries: &[
                    // input
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
                    // template
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
                    // result
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
                    // uniform
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
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("CrossCorrelationMatcher PipelineLayout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let uniform_buffer = ctx.device.create_buffer(&BufferDescriptor {
            label: Some("uniform"),
            size: size_of::<Uniforms>() as _,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader_module = ctx
            .device
            .create_shader_module(include_wgsl!("./shaders/cross_correlation.wgsl"));
        let pipeline = ctx
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some("CrossCorrelationMatcher Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "main",
                compilation_options: PipelineCompilationOptions::default(),
            });

        CrossCorrelationMatcher {
            ctx,
            input_buffer: None,
            template_buffer: None,
            result_buffer: None,
            staging_buffer: None,
            uniform_buffer,
            bind_group_layout,
            bind_group: None,
            pipeline_layout,
            pipeline,
        }
    }

    fn create_new_bind_group(&self) -> BindGroup {
        self.ctx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("CrossCorrelationMatcher BindGroup"),
            layout: &self.bind_group_layout,
            entries: &[
                // input
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.input_buffer.as_ref().unwrap().as_entire_binding(),
                },
                // template
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.template_buffer.as_ref().unwrap().as_entire_binding(),
                },
                // result
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.result_buffer.as_ref().unwrap().as_entire_binding(),
                },
                // uniform
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        })
    }

    pub fn match_template(
        &mut self,
        image: &ImageBuffer<Luma<f32>, Vec<f32>>,
        template: &ImageBuffer<Luma<f32>, Vec<f32>>,
    ) -> ImageBuffer<Luma<f32>, Vec<f32>> {
        let (result_w, result_h) = (
            image.width() - template.width() + 1,
            image.height() - template.height() + 1,
        );
        let result_buf_sz = (result_w * result_h * size_of::<f32>() as u32) as u64;

        // update buffers
        let update = prepare_buffer_init_with_image(
            &self.ctx,
            &mut self.input_buffer,
            image,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );
        let update = prepare_buffer_init_with_image(
            &self.ctx,
            &mut self.template_buffer,
            template,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
        ) || update;
        let update = prepare_buffer_init_with_size(
            &self.ctx,
            &mut self.result_buffer,
            result_buf_sz,
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
        ) || update;
        let update = prepare_buffer_init_with_size(
            &self.ctx,
            &mut self.staging_buffer,
            result_buf_sz,
            BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        ) || update;

        // update bind_group and uniforms
        if update {
            self.bind_group = Some(self.create_new_bind_group());
            let uniforms = Uniforms {
                image_height: image.height(),
                image_width: image.width(),
                template_height: template.height(),
                template_width: template.width(),
            };
            self.ctx
                .queue
                .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        }

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("encoder"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("compute pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            pass.dispatch_workgroups(
                (result_w as f32 / 16.0).ceil() as u32,
                (result_h as f32 / 16.0).ceil() as u32,
                1,
            );
        }
        encoder.copy_buffer_to_buffer(
            self.result_buffer.as_ref().unwrap(),
            0,
            self.staging_buffer.as_ref().unwrap(),
            0,
            result_buf_sz,
        );

        self.ctx.queue.submit(std::iter::once(encoder.finish()));

        // get output
        let buffer_slice = self.staging_buffer.as_ref().unwrap().slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        self.ctx.device.poll(wgpu::Maintain::Wait);

        pollster::block_on(async {
            let result;

            if let Some(Ok(())) = receiver.receive().await {
                let data = buffer_slice.get_mapped_range();
                result = bytemuck::cast_slice(&data).to_vec();
                drop(data);
                self.staging_buffer.as_ref().unwrap().unmap();
            } else {
                result = vec![0.0; (result_w * result_h) as usize]
            };

            let res = ImageBuffer::from_vec(result_w, result_h, result).unwrap();

            res
        })
    }
}

/// returns true if buffer is updated
fn prepare_buffer_init_with_size(
    ctx: &Context,
    buffer: &mut Option<wgpu::Buffer>,
    size: u64,
    usage: wgpu::BufferUsages,
) -> bool {
    let update = buffer.is_none() || buffer.as_ref().unwrap().size() != size;
    if update {
        *buffer = Some(ctx.device.create_buffer(&BufferDescriptor {
            label: None,
            size,
            usage,
            mapped_at_creation: false,
        }));
    }
    update
}

/// returns true if buffer is updated
fn prepare_buffer_init_with_image(
    ctx: &Context,
    buffer: &mut Option<wgpu::Buffer>,
    image: &ImageBuffer<Luma<f32>, Vec<f32>>,
    usage: wgpu::BufferUsages,
) -> bool {
    let update = buffer.is_none()
        || buffer.as_ref().unwrap().size() != (image.len() * size_of::<f32>()) as _;
    if update {
        *buffer = Some(
            ctx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&image.as_raw()),
                    usage,
                }),
        );
    } else {
        ctx.queue.write_buffer(
            buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&image.as_raw()),
        );
    }
    update
}

#[cfg(test)]
mod test {
    use image::{open, ImageBuffer, Luma};

    use super::CrossCorrelationMatcher;

    #[test]
    fn test_cross_correlation_match() {
        let mut matcher = CrossCorrelationMatcher::new();
        let image = open("../../resources/templates/MUMU-1920x1080/start.png").unwrap();
        let template = open("../../resources/templates/1920x1080/start_start.png").unwrap();

        let res = matcher.match_template(&image.to_luma32f(), &template.to_luma32f());
        let res: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_vec(
            res.width(),
            res.height(),
            res.as_raw().iter().map(|x| (x * 255.0) as u8).collect(),
        )
        .unwrap();
        res.save("./output.png").unwrap();
    }
}
