//! wgpu renderer. What it draws is specified in `docs/specs/canvas-rendering.md`.

mod view;

pub use view::View;

use celframe_core::Stroke;
use celframe_geom::{StrokeMesh, tessellate_stroke};

/// Edges are anti-aliased with this many samples per pixel.
const SAMPLE_COUNT: u32 = 4;
/// Headless images use this format. Bytes come back as RGBA.
const IMAGE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// Position (2 × f32) then premultiplied colour (4 × f32).
const VERTEX_SIZE: u64 = 24;

const SHADER: &str = "
struct Uniforms {
    row_x: vec4<f32>,
    row_y: vec4<f32>,
}
@group(0) @binding(0) var<uniform> u: Uniforms;

struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex fn vs(@location(0) p: vec2<f32>, @location(1) color: vec4<f32>) -> Out {
    let x = u.row_x.x * p.x + u.row_x.y * p.y + u.row_x.z;
    let y = u.row_y.x * p.x + u.row_y.y * p.y + u.row_y.z;
    return Out(vec4<f32>(x, y, 0.0, 1.0), color);
}

@fragment fn fs(input: Out) -> @location(0) vec4<f32> {
    return input.color;
}
";

#[derive(Debug)]
pub enum RenderError {
    /// No GPU, or no software fallback, is available.
    NoAdapter(String),
    NoDevice(String),
    /// The image size is zero or bigger than the GPU allows.
    BadSize {
        width: u32,
        height: u32,
    },
    Readback(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAdapter(e) => write!(f, "no graphics adapter is available: {e}"),
            Self::NoDevice(e) => write!(f, "the graphics device couldn't start: {e}"),
            Self::BadSize { width, height } => {
                write!(f, "can't render a {width} × {height} image")
            }
            Self::Readback(e) => write!(f, "couldn't read the rendered image: {e}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// A rendered image: RGBA, 8 bits per channel, rows top to bottom, premultiplied alpha.
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Image {
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * self.width + x) * 4) as usize;
        [
            self.pixels[i],
            self.pixels[i + 1],
            self.pixels[i + 2],
            self.pixels[i + 3],
        ]
    }
}

/// Where a frame is drawn on its target, and what surrounds the strokes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Canvas {
    /// The whole target texture, in pixels.
    pub target_size: [u32; 2],
    /// The part of the target that shows the scene, as `[x, y, width, height]` in pixels from
    /// the top left. The view's centre is at its middle.
    pub viewport: [u32; 4],
    /// Fills the whole target, around and behind the paper.
    pub backdrop: [u8; 4],
    /// The scene frame, drawn behind the strokes.
    pub paper: Option<Paper>,
}

impl Canvas {
    /// A canvas that uses the whole target, with no paper.
    pub fn full(width: u32, height: u32, backdrop: [u8; 4]) -> Self {
        Self {
            target_size: [width, height],
            viewport: [0, 0, width, height],
            backdrop,
            paper: None,
        }
    }

    /// The viewport, trimmed to fit inside the target.
    fn clamped_viewport(&self) -> [u32; 4] {
        let [tw, th] = self.target_size;
        let [x, y, w, h] = self.viewport;
        let (x, y) = (x.min(tw), y.min(th));
        [x, y, w.min(tw - x), h.min(th - y)]
    }
}

/// A rectangle in scene space, such as the scene frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Paper {
    /// `[min x, min y, max x, max y]` in scene units.
    pub rect: [f64; 4],
    pub color: [u8; 4],
}

impl Paper {
    /// The frame of a scene of this size, centred on the origin as the units spec says.
    pub fn scene_frame(width: u32, height: u32, color: [u8; 4]) -> Self {
        let (w, h) = (f64::from(width) / 2.0, f64::from(height) / 2.0);
        Self {
            rect: [-w, -h, w, h],
            color,
        }
    }
}

/// Draws strokes with wgpu. One renderer holds one GPU device.
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline_layout: wgpu::PipelineLayout,
    shader: wgpu::ShaderModule,
    pipelines: Vec<(wgpu::TextureFormat, wgpu::RenderPipeline)>,
    uniforms: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    mesh: StrokeMesh,
    vertex_bytes: Vec<u8>,
    adapter_name: String,
}

impl Renderer {
    /// Starts a renderer with no window, for tests and offscreen images. Falls back to a
    /// software adapter if there is no GPU.
    pub fn new_headless() -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(request_adapter(&instance, None))?;
        pollster::block_on(Self::from_adapter(&adapter))
    }

    /// Starts a renderer on an adapter, for example one chosen to suit a window's surface.
    pub async fn from_adapter(adapter: &wgpu::Adapter) -> Result<Self, RenderError> {
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("celframe"),
                required_limits: wgpu::Limits::downlevel_defaults()
                    .using_resolution(adapter.limits()),
                ..Default::default()
            })
            .await
            .map_err(|e| RenderError::NoDevice(e.to_string()))?;

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("view"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("strokes"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("strokes"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("view"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("view"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });

        Ok(Self {
            device,
            queue,
            pipeline_layout,
            shader,
            pipelines: Vec::new(),
            uniforms,
            bind_group,
            mesh: StrokeMesh::default(),
            vertex_bytes: Vec::new(),
            adapter_name: adapter.get_info().name,
        })
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Sends recorded work to the GPU.
    pub fn submit(&self, encoder: wgpu::CommandEncoder) {
        self.queue.submit([encoder.finish()]);
    }

    /// Shows a finished window frame.
    pub fn present(&self, frame: wgpu::SurfaceTexture) {
        self.queue.present(frame);
    }

    /// Draws strokes into a new offscreen image and reads its pixels back.
    pub fn render_image<'a>(
        &mut self,
        strokes: impl IntoIterator<Item = &'a Stroke>,
        view: &View,
        canvas: &Canvas,
    ) -> Result<Image, RenderError> {
        let [width, height] = canvas.target_size;
        let max = self.device.limits().max_texture_dimension_2d;
        if width == 0 || height == 0 || width > max || height > max {
            return Err(RenderError::BadSize { width, height });
        }
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("image"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: IMAGE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target = texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.draw(&mut encoder, &target, IMAGE_FORMAT, canvas, strokes, view);

        // Rows in a buffer copy must start on 256-byte boundaries.
        let row_bytes = width * 4;
        let padded_row_bytes = row_bytes.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(padded_row_bytes) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);

        let slice = readback.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| RenderError::Readback(e.to_string()))?;
        receiver
            .recv()
            .map_err(|e| RenderError::Readback(e.to_string()))?
            .map_err(|e| RenderError::Readback(e.to_string()))?;

        let mapped = slice
            .get_mapped_range()
            .map_err(|e| RenderError::Readback(e.to_string()))?;
        let mut pixels = Vec::with_capacity((row_bytes * height) as usize);
        for row in mapped.chunks(padded_row_bytes as usize) {
            pixels.extend_from_slice(&row[..row_bytes as usize]);
        }
        drop(mapped);
        readback.unmap();
        Ok(Image {
            width,
            height,
            pixels,
        })
    }

    /// Records a frame that clears `target` and draws the paper and strokes on it. The window
    /// renderer shares this with headless rendering.
    pub fn draw<'a>(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        format: wgpu::TextureFormat,
        canvas: &Canvas,
        strokes: impl IntoIterator<Item = &'a Stroke>,
        view: &View,
    ) {
        let size = canvas.target_size;
        let viewport = canvas.clamped_viewport();
        self.mesh.clear();
        self.vertex_bytes.clear();
        if let Some(paper) = canvas.paper {
            let [x0, y0, x1, y1] = paper.rect.map(|v| v as f32);
            self.mesh
                .positions
                .extend([[x0, y0], [x1, y0], [x1, y1], [x0, y1]]);
            self.mesh.indices.extend([0, 1, 2, 0, 2, 3]);
            self.push_vertex_colors(0, paper.color);
        }
        let tolerance = view.tolerance();
        for stroke in strokes {
            let first = self.mesh.positions.len();
            tessellate_stroke(stroke, tolerance, &mut self.mesh);
            self.push_vertex_colors(first, stroke.color);
        }

        let matrix = view.clip_matrix([f64::from(viewport[2]), f64::from(viewport[3])]);
        let mut uniform_bytes = Vec::with_capacity(32);
        for row in matrix {
            for value in row.iter().chain(&[0.0]) {
                uniform_bytes.extend_from_slice(&value.to_ne_bytes());
            }
        }
        self.queue.write_buffer(&self.uniforms, 0, &uniform_bytes);

        // wgpu rejects zero-sized buffers, so an empty canvas only clears.
        let buffers = (!self.mesh.indices.is_empty()).then(|| {
            let index_bytes: Vec<u8> = self
                .mesh
                .indices
                .iter()
                .flat_map(|i| i.to_ne_bytes())
                .collect();
            (
                self.buffer(&self.vertex_bytes, wgpu::BufferUsages::VERTEX),
                self.buffer(&index_bytes, wgpu::BufferUsages::INDEX),
            )
        });

        let multisampled = self
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("multisampled"),
                size: wgpu::Extent3d {
                    width: size[0],
                    height: size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default());

        let pipeline = self.pipeline(format).clone();
        let [r, g, b, a] = premultiplied(canvas.backdrop).map(f64::from);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("strokes"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &multisampled,
                depth_slice: None,
                resolve_target: Some(target),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a }),
                    store: wgpu::StoreOp::Discard,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let [vx, vy, vw, vh] = viewport;
        if let Some((vertices, indices)) = buffers.as_ref().filter(|_| vw > 0 && vh > 0) {
            pass.set_viewport(vx as f32, vy as f32, vw as f32, vh as f32, 0.0, 1.0);
            pass.set_scissor_rect(vx, vy, vw, vh);
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, vertices.slice(..));
            pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
            let count = u32::try_from(self.mesh.indices.len()).unwrap_or(u32::MAX);
            pass.draw_indexed(0..count, 0, 0..1);
        }
    }

    /// Writes vertex data for positions from `first` on, all in one colour.
    fn push_vertex_colors(&mut self, first: usize, color: [u8; 4]) {
        let color = premultiplied(color);
        for position in &self.mesh.positions[first..] {
            for value in position.iter().chain(&color) {
                self.vertex_bytes.extend_from_slice(&value.to_ne_bytes());
            }
        }
    }

    fn buffer(&self, bytes: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: bytes.len() as u64,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buffer, 0, bytes);
        buffer
    }

    /// One pipeline per target format, created on first use.
    fn pipeline(&mut self, format: wgpu::TextureFormat) -> &wgpu::RenderPipeline {
        if let Some(index) = self.pipelines.iter().position(|(f, _)| *f == format) {
            return &self.pipelines[index].1;
        }
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("strokes"),
                layout: Some(&self.pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.shader,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: VERTEX_SIZE,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
                    })],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader,
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: SAMPLE_COUNT,
                    ..Default::default()
                },
                multiview_mask: None,
                cache: None,
            });
        self.pipelines.push((format, pipeline));
        &self.pipelines[self.pipelines.len() - 1].1
    }
}

/// Finds a GPU, or a software adapter if there is none.
pub async fn request_adapter(
    instance: &wgpu::Instance,
    surface: Option<&wgpu::Surface<'_>>,
) -> Result<wgpu::Adapter, RenderError> {
    let request = |force_fallback_adapter| wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: surface,
        force_fallback_adapter,
        apply_limit_buckets: false,
    };
    match instance.request_adapter(&request(false)).await {
        Ok(adapter) => Ok(adapter),
        Err(_) => instance
            .request_adapter(&request(true))
            .await
            .map_err(|e| RenderError::NoAdapter(e.to_string())),
    }
}

fn premultiplied(color: [u8; 4]) -> [f32; 4] {
    let alpha = f32::from(color[3]) / 255.0;
    [
        f32::from(color[0]) / 255.0 * alpha,
        f32::from(color[1]) / 255.0 * alpha,
        f32::from(color[2]) / 255.0 * alpha,
        alpha,
    ]
}
