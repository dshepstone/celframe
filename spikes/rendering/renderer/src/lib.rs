//! Spike renderer shared by both prototypes: variable-width strokes, tessellated with lyon.

use bytemuck::{Pod, Zeroable};
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder, StrokeOptions, StrokeTessellator, StrokeVertexConstructor, VertexBuffers,
};

#[cfg(target_arch = "wasm32")]
pub mod web;

/// Room for about 10,000 strokes plus one being drawn.
const MAX_VERTICES: u64 = 6_000_000;
const MAX_INDICES: u64 = 18_000_000;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
}

struct Ctor;
impl StrokeVertexConstructor<Vertex> for Ctor {
    fn new_vertex(&mut self, vertex: lyon::tessellation::StrokeVertex) -> Vertex {
        Vertex {
            position: vertex.position().to_array(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Sample {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
}

const WGSL: &str = "
@vertex fn vs(@location(0) p: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(p, 0.0, 1.0);
}
@fragment fn fs() -> @location(0) vec4<f32> {
    return vec4<f32>(0.12, 0.11, 0.09, 1.0);
}
";

fn tessellate(samples: &[Sample], out: &mut VertexBuffers<Vertex, u32>) {
    let mut builder = Path::builder_with_attributes(1);
    builder.begin(point(samples[0].x, samples[0].y), &[samples[0].pressure * 0.006]);
    for s in &samples[1..] {
        builder.line_to(point(s.x, s.y), &[s.pressure * 0.006]);
    }
    builder.end(false);
    let path = builder.build();
    let options = StrokeOptions::default()
        .with_variable_line_width(0)
        .with_tolerance(0.0005);
    StrokeTessellator::new()
        .tessellate_path(&path, &options, &mut BuffersBuilder::new(out, Ctor))
        .expect("tessellation failed");
}

/// A repeatable pseudo-random wavy stroke, so every run draws the same background.
fn background_stroke(seed: u32) -> Vec<Sample> {
    let f = seed as f32;
    let (x0, y0) = ((f * 0.618).fract() * 1.8 - 0.9, (f * 0.381).fract() * 1.8 - 0.9);
    (0..32)
        .map(|i| {
            let t = i as f32 / 31.0;
            Sample {
                x: x0 + t * 0.2,
                y: y0 + (t * 9.0 + f).sin() * 0.03,
                pressure: 0.4 + 0.6 * (t * 3.14).sin(),
            }
        })
        .collect()
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    vertex_count: u32,
    index_count: u32,
    last_sample: Option<Sample>,
    pub backend: String,
    pub adapter_name: String,
}

impl Renderer {
    pub async fn new(
        instance: wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
        power_preference: wgpu::PowerPreference,
    ) -> Result<Self, String> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .map_err(|e| format!("no adapter: {e}"))?;
        let info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_limits: adapter.limits(),
                ..Default::default()
            })
            .await
            .map_err(|e| format!("no device: {e}"))?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 1,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 8,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(format.into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let buffer = |size, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size,
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let vertices = buffer(MAX_VERTICES * 8, wgpu::BufferUsages::VERTEX);
        let indices = buffer(MAX_INDICES * 4, wgpu::BufferUsages::INDEX);

        Ok(Self {
            device,
            queue,
            surface,
            config,
            pipeline,
            vertices,
            indices,
            vertex_count: 0,
            index_count: 0,
            last_sample: None,
            backend: format!("{:?}", info.backend),
            adapter_name: info.name,
        })
    }

    fn append(&mut self, samples: &[Sample]) {
        let mut mesh = VertexBuffers::new();
        tessellate(samples, &mut mesh);
        if u64::from(self.vertex_count) + mesh.vertices.len() as u64 > MAX_VERTICES
            || u64::from(self.index_count) + mesh.indices.len() as u64 > MAX_INDICES
        {
            return;
        }
        let base = self.vertex_count;
        let shifted: Vec<u32> = mesh.indices.iter().map(|i| i + base).collect();
        self.queue.write_buffer(
            &self.vertices,
            u64::from(self.vertex_count) * 8,
            bytemuck::cast_slice(&mesh.vertices),
        );
        self.queue.write_buffer(
            &self.indices,
            u64::from(self.index_count) * 4,
            bytemuck::cast_slice(&shifted),
        );
        self.vertex_count += mesh.vertices.len() as u32;
        self.index_count += shifted.len() as u32;
    }

    /// Replaces everything with `count` background strokes.
    pub fn load_background(&mut self, count: u32) {
        self.vertex_count = 0;
        self.index_count = 0;
        self.last_sample = None;
        for seed in 0..count {
            self.append(&background_stroke(seed));
        }
    }

    pub fn pen_up(&mut self) {
        self.last_sample = None;
    }

    /// Adds a pen sample, drawing the segment from the previous sample.
    pub fn add_sample(&mut self, sample: Sample) {
        if let Some(previous) = self.last_sample {
            self.append(&[previous, sample]);
        }
        self.last_sample = Some(sample);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    /// Draws and presents one frame. Returns false if the surface had no frame to give.
    pub fn render(&mut self) -> bool {
        use wgpu::CurrentSurfaceTexture as T;
        let frame = match self.surface.get_current_texture() {
            T::Success(frame) | T::Suboptimal(frame) => frame,
            T::Outdated | T::Lost => {
                self.surface.configure(&self.device, &self.config);
                return false;
            }
            _ => return false,
        };
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.98,
                            g: 0.97,
                            b: 0.94,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..self.index_count, 0, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        true
    }

    /// Gives wgpu a chance to run completion callbacks. Only the WebGL path needs this.
    pub fn poll(&self) {
        let _ = self.device.poll(wgpu::PollType::Poll);
    }

    /// Calls back when the GPU has finished all submitted work.
    pub fn on_done(&self, callback: impl FnOnce() + Send + 'static) {
        self.queue.on_submitted_work_done(callback);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn wait_for_gpu(&self) {
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
    }
}
