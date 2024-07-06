use std::marker::PhantomData;

use bytemuck::Pod;
use wgpu::{BindGroupEntry, BindGroupLayoutEntry};

pub struct Context {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Context {
    pub async fn new() -> Self {
        // Instantiates instance of WebGPU
        let instance = wgpu::Instance::default();

        // `request_adapter` instantiates the general connection to the GPU
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

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

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
}

pub struct BindGroupLayoutEntriesBuilder {
    bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindGroupLayoutEntriesBuilder {
    pub fn new() -> Self {
        Self {
            bind_group_layout_entries: Vec::new(),
        }
    }

    pub fn add(mut self, bind_group_layout_entry: wgpu::BindGroupLayoutEntry) -> Self {
        self.bind_group_layout_entries.push(bind_group_layout_entry);
        self
    }

    pub fn add_buffer(self, buffer_binding_type: wgpu::BufferBindingType) -> Self {
        let binding = self.bind_group_layout_entries.len() as u32;

        self.add(BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: buffer_binding_type,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        })
    }

    pub fn build(&self) -> &[wgpu::BindGroupLayoutEntry] {
        &self.bind_group_layout_entries
    }
}

pub struct BindGroupEntriesBuilder<'a> {
    bind_group_entries: Vec<wgpu::BindGroupEntry<'a>>,
}

impl<'a> BindGroupEntriesBuilder<'a> {
    pub fn new() -> Self {
        Self {
            bind_group_entries: Vec::new(),
        }
    }

    pub fn add(mut self, bind_group_layout_entry: wgpu::BindGroupEntry<'a>) -> Self {
        self.bind_group_entries.push(bind_group_layout_entry);
        self
    }

    pub fn add_buffer(self, buffer: &'a wgpu::Buffer) -> Self {
        let binding = self.bind_group_entries.len() as u32;

        self.add(BindGroupEntry {
            binding,
            resource: buffer.as_entire_binding(),
        })
    }

    pub fn build(&self) -> &[wgpu::BindGroupEntry] {
        &self.bind_group_entries
    }
}

pub trait GpuTask {
    /// 0 is the binding for result buffer
    fn build_bind_group_layout_entries(
        &self,
        bind_group_layout_entry_builder: BindGroupLayoutEntriesBuilder,
    ) -> BindGroupLayoutEntriesBuilder;

    /// 0 is the binding for result buffer
    fn build_bind_group_entries<'a>(
        &'a self,
        bind_group_entry_builder: BindGroupEntriesBuilder<'a>,
    ) -> BindGroupEntriesBuilder<'a>;

    fn cs_module(&self, device: &wgpu::Device) -> wgpu::ShaderModule;

    fn prepare(&self, queue: &wgpu::Queue) {}

    fn exec(&self, cpass: &mut wgpu::ComputePass);
}

pub struct GpuTaskWrapper<T> {
    context: Context,
    staging_buffer: wgpu::Buffer,
    result_buffer: wgpu::Buffer,
    result_size_in_byte: u64,
    phantom: PhantomData<T>,
    pipeline: wgpu::ComputePipeline,
    bindgroup: wgpu::BindGroup,

    task: Box<dyn GpuTask>,
}

impl<T: Pod> GpuTaskWrapper<T> {
    pub async fn new(context: Context, result_size_in_byte: u64, task: Box<dyn GpuTask>) -> Self {
        // Instantiates buffer without data.
        // `usage` of buffer specifies how it can be used:
        //   `BufferUsages::MAP_READ` allows it to be read (outside the shader).
        //   `BufferUsages::COPY_DST` allows it to be the destination of the copy.
        let staging_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: result_size_in_byte,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let result_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: result_size_in_byte,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let entries_builder = BindGroupLayoutEntriesBuilder::new()
            .add_buffer(wgpu::BufferBindingType::Storage { read_only: false });

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: task
                        .build_bind_group_layout_entries(entries_builder)
                        .build(),
                });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Pipeline layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    ..Default::default()
                });

        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                module: &task.cs_module(&context.device),
                entry_point: "main",
                compilation_options: Default::default(),
            });

        let entries_builder = BindGroupEntriesBuilder::new().add_buffer(&result_buffer);
        let bindgroup = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: task.build_bind_group_entries(entries_builder).build(),
            });

        Self {
            context,
            staging_buffer,
            result_buffer,
            result_size_in_byte,
            phantom: PhantomData::default(),
            task,
            pipeline,
            bindgroup,
        }
    }

    pub async fn exec(&self) -> Option<Vec<T>> {
        let mut encoder = self
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.task.prepare(&self.context.queue);
        // A command encoder executes one or many pipelines.
        // It is to WebGPU what a command buffer is to Vulkan.
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &self.bindgroup, &[]);
            cpass.insert_debug_marker("compute");

            self.task.exec(&mut cpass);
        }
        // ? get result

        // Sets adds copy operation to command encoder.
        // Will copy data from storage buffer on GPU to staging buffer on CPU.
        encoder.copy_buffer_to_buffer(
            &self.result_buffer,
            0,
            &self.staging_buffer,
            0,
            self.result_size_in_byte,
        );
        // Submits command encoder for processing
        self.context.queue.submit(Some(encoder.finish()));
        self.wait_for_result().await
    }

    pub async fn wait_for_result(&self) -> Option<Vec<T>> {
        // Note that we're not calling `.await` here.
        let buffer_slice = self.staging_buffer.slice(..);
        // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        // Poll the device in a blocking manner so that our future resolves.
        // In an actual application, `device.poll(...)` should
        // be called in an event loop or on another thread.
        self.context
            .device
            .poll(wgpu::Maintain::wait())
            .panic_on_timeout();

        // Awaits until `buffer_future` can be read from
        if let Ok(Ok(())) = receiver.recv_async().await {
            // Gets contents of buffer
            let data = buffer_slice.get_mapped_range();
            // Since contents are got in bytes, this converts these bytes back to u32
            let result: Vec<T> = bytemuck::cast_slice(&data).to_vec();

            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(data);
            self.staging_buffer.unmap(); // Unmaps buffer from memory
                                         // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                         //   delete myPointer;
                                         //   myPointer = NULL;
                                         // It effectively frees the memory

            // Returns data from buffer
            // let result = Array2::from_shape_vec((result_height, result_width), result).unwrap();
            Some(result)
        } else {
            panic!("failed to run compute on gpu!")
        }
    }
}
