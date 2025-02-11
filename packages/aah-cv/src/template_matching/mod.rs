use std::{
    fmt::Display,
    sync::{Arc, Mutex, OnceLock},
};

use bytemuck::{Pod, Zeroable};
use image::{ImageBuffer, Luma};
use serde::{Deserialize, Serialize};
use wgpu::{
    include_wgsl, util::DeviceExt, BindGroup, BindGroupDescriptor, BindGroupLayoutDescriptor,
    BufferDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor,
    PipelineLayoutDescriptor,
};

use crate::gpu::Context;

pub struct Match {
    pub location: (u32, u32),
    pub value: f32,
}

pub fn find_matches(
    input: &ImageBuffer<Luma<f32>, Vec<f32>>,
    template_width: u32,
    template_height: u32,
    method: MatchTemplateMethod,
    threshold: f32,
) -> Vec<Match> {
    let mut matches: Vec<Match> = Vec::new();

    let input_width = input.width();
    let input_height = input.height();

    for y in 0..input_height {
        for x in 0..input_width {
            let value = input.get_pixel(x, y).0[0];

            let ok = if matches!(
                method,
                MatchTemplateMethod::SumOfSquaredDifference
                    | MatchTemplateMethod::SumOfSquaredDifferenceNormed
            ) {
                value < threshold
            } else {
                value > threshold
            };
            if ok {
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

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MatchTemplateMethod {
    SumOfSquaredDifference,
    SumOfSquaredDifferenceNormed,
    CrossCorrelation,
    CrossCorrelationNormed,
    CorrelationCoefficient,
    CorrelationCoefficientNormed,
}

impl Display for MatchTemplateMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MatchTemplateMethod::SumOfSquaredDifference => "sqdiff",
            MatchTemplateMethod::SumOfSquaredDifferenceNormed => "sqdiff_normed",
            MatchTemplateMethod::CrossCorrelation => "ccorr",
            MatchTemplateMethod::CrossCorrelationNormed => "ccorr_normed",
            MatchTemplateMethod::CorrelationCoefficient => "ccoeff",
            MatchTemplateMethod::CorrelationCoefficientNormed => "ccoeff_normed",
        };
        f.write_str(s)
    }
}

pub fn match_template(
    image: &ImageBuffer<Luma<f32>, Vec<f32>>,
    template: &ImageBuffer<Luma<f32>, Vec<f32>>,
    method: MatchTemplateMethod,
    padding: bool,
) -> ImageBuffer<Luma<f32>, Vec<f32>> {
    let mut matcher = matcher().lock().unwrap();
    matcher.match_template(image, template, method, padding)
}

/// internal
fn matcher() -> &'static Arc<Mutex<Matcher>> {
    static MATCHER: OnceLock<Arc<Mutex<Matcher>>> = OnceLock::new();
    MATCHER.get_or_init(|| Arc::new(Mutex::new(Matcher::new())))
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Uniforms {
    image_width: u32,
    image_height: u32,
    template_width: u32,
    template_height: u32,
}

struct Matcher {
    ctx: Context,

    input_buffer: Option<wgpu::Buffer>,
    template_buffer: Option<wgpu::Buffer>,
    result_buffer: Option<wgpu::Buffer>,
    staging_buffer: Option<wgpu::Buffer>,
    uniform_buffer: wgpu::Buffer,

    bind_group_layout: wgpu::BindGroupLayout,
    // pipeline_layout: wgpu::PipelineLayout,
    bind_group: Option<wgpu::BindGroup>,
    pipeline_ccorr: wgpu::ComputePipeline,
    pipeline_ccorr_normed: wgpu::ComputePipeline,
    pipeline_sqdiff: wgpu::ComputePipeline,
    pipeline_sqdiff_normed: wgpu::ComputePipeline,
    pipeline_ccoeff: wgpu::ComputePipeline,
    pipeline_ccoeff_normed: wgpu::ComputePipeline,
}

impl Matcher {
    fn new() -> Self {
        let ctx = pollster::block_on(Context::new());
        let Context { device, .. } = &ctx;

        let bind_group_layout = ctx
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Matcher BindGroupLayout"),
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
                label: Some("Matcher PipelineLayout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let uniform_buffer = ctx.device.create_buffer(&BufferDescriptor {
            label: Some("uniform"),
            size: size_of::<Uniforms>() as _,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader_module = device.create_shader_module(include_wgsl!("./shaders/shader.wgsl"));
        let pipeline_ccorr = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Cross Correlation Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main_ccorr"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let pipeline_ccorr_normed =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Cross Correlation Normed Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: Some("main_ccorr_normed"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        let pipeline_sqdiff = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Sum of Squared Difference Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main_sqdiff"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let pipeline_sqdiff_normed =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Sum of Squared Difference Normed Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: Some("main_sqdiff_normed"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        let pipeline_ccoeff = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Correlation Coefficient Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main_ccoeff"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let pipeline_ccoeff_normed =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Correlation Coefficient Normed Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: Some("main_ccoeff_normed"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        Matcher {
            ctx,
            input_buffer: None,
            template_buffer: None,
            result_buffer: None,
            staging_buffer: None,
            uniform_buffer,
            bind_group_layout,
            bind_group: None,
            // pipeline_layout,
            pipeline_ccorr,
            pipeline_ccorr_normed,
            pipeline_sqdiff,
            pipeline_sqdiff_normed,
            pipeline_ccoeff,
            pipeline_ccoeff_normed,
        }
    }

    fn create_new_bind_group(&self) -> BindGroup {
        // println!("input buffer size: {:?}", self.input_buffer.as_ref().unwrap().size());
        // println!("template buffer size: {:?}", self.template_buffer.as_ref().unwrap().size());
        // println!("result buffer size: {:?}", self.result_buffer.as_ref().unwrap().size());
        // println!("staging buffer size: {:?}", self.staging_buffer.as_ref().unwrap().size());
        self.ctx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Matcher BindGroup"),
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

    fn match_template(
        &mut self,
        image: &ImageBuffer<Luma<f32>, Vec<f32>>,
        template: &ImageBuffer<Luma<f32>, Vec<f32>>,
        match_method: MatchTemplateMethod,
        padding: bool,
    ) -> ImageBuffer<Luma<f32>, Vec<f32>> {
        let (image, template) = if matches!(
            match_method,
            MatchTemplateMethod::CorrelationCoefficient
                | MatchTemplateMethod::CorrelationCoefficientNormed
        ) {
            let avg_kernel = ImageBuffer::from_pixel(
                template.width(),
                template.height(),
                Luma([1.0 / (template.width() * template.height()) as f32]),
            );
            let avg_image = self.match_template(
                image,
                &avg_kernel,
                MatchTemplateMethod::CrossCorrelation,
                true,
            );
            let avg_template = self.match_template(
                template,
                &avg_kernel,
                MatchTemplateMethod::CrossCorrelation,
                true,
            );

            let image = ImageBuffer::from_vec(
                image.width(),
                image.height(),
                image
                    .as_raw()
                    .iter()
                    .zip(avg_image.as_raw().iter())
                    .map(|(v, avg)| v - avg)
                    .collect(),
            )
            .unwrap();
            let template = ImageBuffer::from_vec(
                template.width(),
                template.height(),
                template
                    .as_raw()
                    .iter()
                    .zip(avg_template.as_raw().iter())
                    .map(|(v, avg)| v - avg)
                    .collect(),
            )
            .unwrap();

            (image, template)
        } else {
            (image.clone(), template.clone())
        };
        let image = if padding {
            let padded_image = ImageBuffer::from_fn(
                image.width() + template.width() - 1,
                image.height() + template.height() - 1,
                |x, y| {
                    if x >= image.width() || y >= image.height() {
                        Luma([0.0])
                    } else {
                        *image.get_pixel(x, y)
                    }
                },
            );
            padded_image
        } else {
            image.clone()
        };
        let image = &image;
        let template = &template;

        let (result_w, result_h) = (
            image.width() - template.width() + 1,
            image.height() - template.height() + 1,
        );
        let result_buf_sz = (result_w * result_h * size_of::<f32>() as u32) as u64;

        // update buffers
        let update = [
            prepare_buffer_init_with_image(
                &self.ctx,
                &mut self.input_buffer,
                image,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            ),
            prepare_buffer_init_with_image(
                &self.ctx,
                &mut self.template_buffer,
                template,
                BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ),
            prepare_buffer_init_with_size(
                &self.ctx,
                &mut self.result_buffer,
                result_buf_sz,
                BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            ),
            prepare_buffer_init_with_size(
                &self.ctx,
                &mut self.staging_buffer,
                result_buf_sz,
                BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            ),
        ]
        .iter()
        .any(|x| *x);

        // update bind_group and uniforms
        if update {
            self.bind_group = Some(self.create_new_bind_group());
            // let template_sq_sum = template.as_raw().iter().map(|x| x * x).sum::<f32>();
            let uniforms = Uniforms {
                image_height: image.height(),
                image_width: image.width(),
                template_height: template.height(),
                template_width: template.width(),
                // template_sq_sum,
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
            pass.set_pipeline(match match_method {
                MatchTemplateMethod::CrossCorrelation => &self.pipeline_ccorr,
                MatchTemplateMethod::CrossCorrelationNormed => &self.pipeline_ccorr_normed,
                MatchTemplateMethod::SumOfSquaredDifference => &self.pipeline_sqdiff,
                MatchTemplateMethod::SumOfSquaredDifferenceNormed => &self.pipeline_sqdiff_normed,
                MatchTemplateMethod::CorrelationCoefficient => &self.pipeline_ccoeff,
                MatchTemplateMethod::CorrelationCoefficientNormed => &self.pipeline_ccoeff_normed,
            });
            pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            pass.dispatch_workgroups(
                (result_w as f32 / 8.0).ceil() as u32,
                (result_h as f32 / 8.0).ceil() as u32,
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
        || buffer.as_ref().unwrap().size() != (image.as_raw().len() * size_of::<f32>()) as _;
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
    use crate::utils::save_luma32f;

    use super::*;
    use std::{error::Error, fs, path::Path, time::Instant};

    #[test]
    fn foo() -> Result<(), Box<dyn Error>> {
        let image = image::open("./assets/in_battle.png")?;
        let template = image::open("./assets/battle_deploy-card-cost1.png")?;
        fs::create_dir_all("./assets/output")?;

        let image = image.to_luma32f();
        save_luma32f(&image, "./assets/output/grey.png", false);
        let image = ImageBuffer::from_fn(image.width(), image.height(), |x, y| {
            let mut sum = 0.0;
            let mut cnt = 0;
            for i in x..(x + template.width()).min(image.width()) {
                for j in y..(y + template.height()).min(image.height()) {
                    sum += image.get_pixel(i, j).0[0];
                    cnt += 1;
                }
            }
            // println!("{sum}/{cnt}");
            // println!("{} {}", image.get_pixel(x, y).0[0], sum / cnt as f32);
            Luma([(sum / cnt as f32)])
        });
        save_luma32f(&image, "./assets/output/avg.png", false);
        Ok(())
    }

    #[test]
    fn test_template_matching() -> Result<(), Box<dyn Error>> {
        let image = image::open("./assets/in_battle.png")?;
        let template = image::open("./assets/battle_deploy-card-cost1.png")?;
        fs::create_dir_all("./assets/output")?;

        for method in [
            MatchTemplateMethod::SumOfSquaredDifference,
            MatchTemplateMethod::SumOfSquaredDifferenceNormed,
            MatchTemplateMethod::CrossCorrelation,
            MatchTemplateMethod::CrossCorrelationNormed,
            MatchTemplateMethod::CorrelationCoefficient,
            MatchTemplateMethod::CorrelationCoefficientNormed,
        ] {
            println!("matching using {}...", method);
            let t = Instant::now();
            let res = match_template(&image.to_luma32f(), &template.to_luma32f(), method, false);
            println!("cost: {:?}", t.elapsed());
            save_luma32f(
                &res,
                format!("./assets/output/{method}.png"),
                matches!(
                    method,
                    MatchTemplateMethod::SumOfSquaredDifference
                        | MatchTemplateMethod::CrossCorrelation
                        | MatchTemplateMethod::CorrelationCoefficient
                ),
            );
        }

        Ok(())
    }

    #[test]
    fn test_btn_matching() -> Result<(), Box<dyn Error>> {
        let images = ["in_battle", "1-4_deploying", "1-4_deploying_direction"].map(|name| {
            (
                name.to_string(),
                image::open(format!("./assets/{name}.png")).unwrap(),
            )
        });
        let dir = Path::new("./assets/output/battle_pause");
        let template = image::open("./assets/battle_pause.png")?;
        fs::create_dir_all(&dir)?;

        for method in [
            MatchTemplateMethod::SumOfSquaredDifference,
            MatchTemplateMethod::SumOfSquaredDifferenceNormed,
            MatchTemplateMethod::CrossCorrelation,
            MatchTemplateMethod::CrossCorrelationNormed,
            MatchTemplateMethod::CorrelationCoefficient,
            MatchTemplateMethod::CorrelationCoefficientNormed,
        ] {
            for (name, image) in images.iter() {
                println!("matching using {}...", method);
                let t = Instant::now();
                let res =
                    match_template(&image.to_luma32f(), &template.to_luma32f(), method, false);
                println!("cost: {:?}", t.elapsed());
                save_luma32f(
                    &res,
                    dir.join(format!("{method}-{name}.png")),
                    matches!(
                        method,
                        MatchTemplateMethod::SumOfSquaredDifference
                            | MatchTemplateMethod::CrossCorrelation
                            | MatchTemplateMethod::CorrelationCoefficient
                    ),
                );
            }
        }

        Ok(())
    }
}
