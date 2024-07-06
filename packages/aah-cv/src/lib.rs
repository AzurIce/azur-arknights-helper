//! GPU-accelerated template matching.
//!
//! Faster alternative to [imageproc::template_matching](https://docs.rs/imageproc/latest/imageproc/template_matching/index.html).

#![deny(clippy::all)]
// #![allow(dead_code)]
// #![allow(unused_variables)]

pub mod ccoeff_normed;
pub mod convolve;
pub mod fft;
pub mod gpu;
pub mod template_matching;
pub mod types;
pub mod utils;

use ccoeff_normed::CcoeffNormedMatcher;
use gpu::Context;
use image::{ImageBuffer, Luma};
use imageproc::template_matching::Extremes;
use std::{
    borrow::Cow,
    mem::size_of,
    ops::{Add, Div, Mul},
};
use types::Image;
use utils::{image_mean, square_sum};
use wgpu::{util::DeviceExt, PipelineCompilationOptions};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MatchTemplateMethod {
    SumOfAbsoluteErrors,
    SumOfSquaredErrors,
    CrossCorrelation,
    CCOEFF,
    CCOEFF_NORMED,
}

/// Slides a template over the input and scores the match at each point using the requested method.
///
/// This is a shorthand for:
/// ```ignore
/// let mut matcher = TemplateMatcher::new();
/// matcher.match_template(input, template, method);
/// matcher.wait_for_result().unwrap()
/// ```
/// You can use  [find_extremes] to find minimum and maximum values, and their locations in the result image.
pub fn match_template<'a>(
    input: &ImageBuffer<Luma<f32>, Vec<f32>>,
    template: &ImageBuffer<Luma<f32>, Vec<f32>>,
    method: MatchTemplateMethod,
) -> Image<'static> {
    match method {
        MatchTemplateMethod::CCOEFF => ccoeff(input, template, false),
        MatchTemplateMethod::CCOEFF_NORMED => ccoeff(input, template, true),
        _ => {
            let mut matcher = TemplateMatcher::new();
            matcher.match_template(input.into(), template.into(), method, true);
            matcher.wait_for_result().unwrap()
        }
    }
}

#[cfg(test)]
mod test {
    use image::{ImageBuffer, Luma};

    use crate::ccoeff;

    #[test]
    fn test_ccoeff() {
        let input = ImageBuffer::from_fn(7, 7, |x, y| Luma([x as f32 + y as f32]));
        let template = ImageBuffer::from_fn(2, 2, |x, y| Luma([x as f32 + y as f32]));
        let res = ccoeff(&input, &template, false);
        println!("{:?}", res);
        let res_normed = ccoeff(&input, &template, true);
        println!("{:?}", res_normed);
    }
}

pub fn ccoeff_normed(
    input: &ImageBuffer<Luma<f32>, Vec<f32>>,
    template: &ImageBuffer<Luma<f32>, Vec<f32>>,
) -> ImageBuffer<Luma<f32>, Vec<f32>> {
    let matcher = CcoeffNormedMatcher::new(&input, &template, true);
    let res = matcher.run();
    res
}

pub fn ccoeff<'a>(
    input: &ImageBuffer<Luma<f32>, Vec<f32>>,
    template: &ImageBuffer<Luma<f32>, Vec<f32>>,
    normed: bool,
) -> Image<'static> {
    let mask = ImageBuffer::from_pixel(template.width(), template.height(), Luma([1.0f32]));
    let i: Image = input.into();
    let m: Image = (&mask).into();
    let t: Image = (template).into();

    let avg_core = ImageBuffer::from_pixel(
        template.width(),
        template.height(),
        Luma([1.0 / (template.width() * template.height()) as f32]),
    );
    let avg_i = ccorr(input, &avg_core, true);
    let ic = i.clone() - avg_i;

    // T' * M where T' = M * (T - 1/sum(M)*sum(M*T))
    let tc = t.clone() - (t.clone() * m.clone()).sum() / m.sum();

    let ccorr_i_tcm = ccorr(i.clone(), tc.clone() * m.clone(), true);
    let ccorr_i_m = ccorr(i.clone(), m.clone(), true);

    // // CCorr(I', T') = CCorr(I, T'*M) - sum(T'*M)/sum(M)*CCorr(I, M)
    // let res = ccorr_i_tcm - (tc.clone() * m.clone()).sum() / m.sum() * ccorr_i_m.clone();
    let res = ccorr(ic, tc.clone(), true);

    if normed {
        // norm(T')
        let norm_templ = tc.square().sum().sqrt();
        // norm(I') = sqrt{ CCorr(I^2, M^2) - 2*CCorr(I, M^2)/sum(M)*CCorr(I, M)
        //                  + sum(M^2)*CCorr(I, M)^2/sum(M)^2 }
        //          = sqrt{ CCorr(I^2, M^2)
        //                  + CCorr(I, M)/sum(M)*{ sum(M^2) / sum(M) * CCorr(I,M)
        //                  - 2 * CCorr(I, M^2) } }
        let i_sq = i.square();
        let m_sq = m.square();
        let ccorr_i_sq_m_sq = ccorr(i_sq.clone(), m_sq.clone(), true);
        let ccorr_i_m_sq = ccorr(i.clone(), m_sq.clone(), true);
        let norm_input = ccorr_i_sq_m_sq
            + ccorr_i_m.clone() / m.sum() * (m_sq.sum() / m.sum() * ccorr_i_m - 2.0 * ccorr_i_m_sq);
        let norm_input = norm_input.sqrt();

        res / (norm_input * norm_templ).replace_zero(1.0)
    } else {
        res
    }
}

pub fn ccorr<'a>(
    input: impl Into<Image<'a>>,
    template: impl Into<Image<'a>>,
    padding: bool,
) -> Image<'static> {
    let mut matcher = TemplateMatcher::new();
    matcher.match_template(
        input.into(),
        template.into(),
        MatchTemplateMethod::CrossCorrelation,
        padding,
    );
    matcher.wait_for_result().unwrap()
}

pub struct Match {
    pub location: (u32, u32),
    pub value: f32,
}

pub fn find_matches(
    input: &Image<'_>,
    template_width: u32,
    template_height: u32,
    threshold: f32,
) -> Vec<Match> {
    let mut matches: Vec<Match> = Vec::new();

    let input_width = input.width;
    let input_height = input.height;

    for y in 0..input_height {
        for x in 0..input_width {
            let idx = (y * input.width) + x;
            let value = input.data[idx as usize];

            if value < threshold {
                if let Some(m) = matches.iter_mut().rev().find(|m| {
                    ((m.location.0 as i32 - x as i32).abs() as u32) < template_width
                        && ((m.location.1 as i32 - y as i32).abs() as u32) < template_height
                }) {
                    if value > m.value {
                        m.location = (x, y);
                        m.value = value;
                    }
                    continue;
                } else {
                    matches.push(Match {
                        location: (x, y),
                        value,
                    });
                }
            }
        }
    }

    matches
}

/// Finds the smallest and largest values and their locations in an image.
pub fn find_extremes(input: &Image<'_>) -> Extremes<f32> {
    let mut min_value = f32::MAX;
    let mut min_value_location = (0, 0);
    let mut max_value = f32::MIN;
    let mut max_value_location = (0, 0);

    for y in 0..input.height {
        for x in 0..input.width {
            let idx = (y * input.width) + x;
            let value = input.data[idx as usize];

            if value < min_value {
                min_value = value;
                min_value_location = (x, y);
            }

            if value > max_value {
                max_value = value;
                max_value_location = (x, y);
            }
        }
    }

    Extremes {
        min_value,
        max_value,
        min_value_location,
        max_value_location,
    }
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
    ctx: gpu::Context,
    shader: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,

    last_pipeline: Option<wgpu::ComputePipeline>,
    last_method: Option<MatchTemplateMethod>,

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
        let ctx = pollster::block_on(Context::new());

        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("../shaders/matching.wgsl"));

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

        let uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform_buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: size_of::<ShaderUniforms>() as _,
            mapped_at_creation: false,
        });

        Self {
            ctx,
            shader,
            pipeline_layout,
            bind_group_layout,
            last_pipeline: None,
            last_method: None,
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
    pub fn wait_for_result(&mut self) -> Option<Image<'static>> {
        if !self.matching_ongoing {
            return None;
        }
        self.matching_ongoing = false;

        let (result_width, result_height) = self.last_result_size;

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
                result = vec![0.0; (result_width * result_height) as usize]
            };

            Some(Image::new(result, result_width as _, result_height as _))
        })
    }

    /// Slides a template over the input and scores the match at each point using the requested method.
    /// To get the result of the matching, call [wait_for_result].
    /// Anchor on top left (0, 0)
    pub fn match_template<'a>(
        &mut self,
        input: Image<'a>,
        template: Image<'a>,
        method: MatchTemplateMethod,
        padding: bool,
    ) {
        if self.matching_ongoing {
            // Discard previous result if not collected.
            self.wait_for_result();
        }

        if self.last_pipeline.is_none() || self.last_method != Some(method) {
            self.last_method = Some(method);

            let entry_point = match method {
                MatchTemplateMethod::SumOfAbsoluteErrors => "main_sae",
                MatchTemplateMethod::SumOfSquaredErrors => "main_sse",
                MatchTemplateMethod::CrossCorrelation => "main_cc",
                _ => panic!("not implemented yet"),
            };

            self.last_pipeline = Some(self.ctx.device.create_compute_pipeline(
                &wgpu::ComputePipelineDescriptor {
                    label: None,
                    layout: Some(&self.pipeline_layout),
                    module: &self.shader,
                    entry_point,
                    compilation_options: PipelineCompilationOptions {
                        ..Default::default()
                    },
                },
            ));
        }

        let mut buffers_changed = false;

        let input = if padding {
            let padded_w = input.width + template.width - 1;
            let padded_h = input.height + template.height - 1;

            let mut padded_input = vec![0.0; padded_w as usize * padded_h as usize];
            for y in 0..input.height {
                for x in 0..input.width {
                    let idx = (y * input.width) + x;
                    let padded_idx = (y * padded_w) + x;
                    padded_input[padded_idx as usize] = input.data[idx as usize];
                }
            }
            Image::new(padded_input, padded_w, padded_h)
        } else {
            input
        };

        let input_size = (input.width, input.height);
        if self.input_buffer.is_none() || self.last_input_size != input_size {
            buffers_changed = true;

            self.last_input_size = input_size;

            self.input_buffer = Some(self.ctx.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("input_buffer"),
                    contents: bytemuck::cast_slice(&input.data),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                },
            ));
        } else {
            self.ctx.queue.write_buffer(
                self.input_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&input.data),
            );
        }

        let template_size = (template.width, template.height);
        if self.template_buffer.is_none() || self.last_template_size != template_size {
            self.ctx.queue.write_buffer(
                &self.uniform_buffer,
                0,
                bytemuck::cast_slice(&[ShaderUniforms {
                    input_width: input.width,
                    input_height: input.height,
                    template_width: template.width,
                    template_height: template.height,
                }]),
            );
            buffers_changed = true;

            self.last_template_size = template_size;

            self.template_buffer = Some(self.ctx.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("template_buffer"),
                    contents: bytemuck::cast_slice(&template.data),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                },
            ));
        } else {
            self.ctx.queue.write_buffer(
                self.template_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&template.data),
            );
        }

        let res_w = input.width - template.width + 1;
        let res_h = input.height - template.height + 1;
        let res_buf_sz = (res_w * res_h) as u64 * size_of::<f32>() as u64;

        if buffers_changed {
            self.last_result_size = (res_w, res_h);

            self.result_buffer = Some(self.ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("result_buffer"),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                size: res_buf_sz,
                mapped_at_creation: false,
            }));

            self.staging_buffer = Some(self.ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("staging_buffer"),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                size: res_buf_sz,
                mapped_at_creation: false,
            }));

            self.bind_group = Some(
                self.ctx
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &self.bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.input_buffer.as_ref().unwrap().as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: self
                                    .template_buffer
                                    .as_ref()
                                    .unwrap()
                                    .as_entire_binding(),
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
                    }),
            );
        }

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
            compute_pass.set_pipeline(self.last_pipeline.as_ref().unwrap());
            compute_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            compute_pass.dispatch_workgroups(
                (res_w as f32 / 16.0).ceil() as u32,
                (res_h as f32 / 16.0).ceil() as u32,
                1,
            );
        }

        encoder.copy_buffer_to_buffer(
            self.result_buffer.as_ref().unwrap(),
            0,
            self.staging_buffer.as_ref().unwrap(),
            0,
            res_buf_sz,
        );

        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        self.matching_ongoing = true;
    }
}
