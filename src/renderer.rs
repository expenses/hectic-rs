use winit::{
    event_loop::EventLoop,
    window::Window,
};

use cgmath::*;
use crate::{WIDTH, HEIGHT};
use crate::components::{Image, Text};
use zerocopy::*;
use wgpu::util::DeviceExt;

pub struct Renderer {
    swap_chain: wgpu::SwapChain,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Window,
    pipeline: wgpu::RenderPipeline,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    surface: wgpu::Surface,
    bind_group: wgpu::BindGroup,
    glyph_brush: wgpu_glyph::GlyphBrush<(), wgpu_glyph::ab_glyph::FontRef<'static>>,
    square_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    instance_buffer: GpuBuffer<'static, Instance>,
}

impl Renderer {
    pub async fn new(event_loop: &EventLoop<()>) -> (Self, BufferRenderer) {
        let window = Window::new(event_loop).unwrap();

        //#[cfg(feature = "native")]
        //window.set_fullscreen(Some(Fullscreen::Borderless(event_loop.primary_monitor())));

        #[cfg(feature = "wasm")]
        {
            // Going fullscreen seems to crash on the web, so we just use a fixed window size for now.
            window.set_inner_size(winit::dpi::LogicalSize::new(1270.0, 720.0));

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas()))
                        .ok()
                })
                .expect("couldn't append canvas to document body");
        }

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe {
            instance.create_surface(&window)
        };

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
        })
            .await
            .unwrap();
    
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Hectic device"),
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            shader_validation: true,
        }, Some(&std::path::Path::new("trace"))).await.unwrap();

        let vs = wgpu::include_spirv!("shader.vert.spv");
        let vs_module = device.create_shader_module(vs);
    
        let fs = wgpu::include_spirv!("shader.frag.spv");
        let fs_module = device.create_shader_module(fs);
    
        let fonts = vec![
            wgpu_glyph::ab_glyph::FontRef::try_from_slice(include_bytes!("fonts/OldeEnglish.ttf")).unwrap(),
            wgpu_glyph::ab_glyph::FontRef::try_from_slice(include_bytes!("fonts/TinyUnicode.ttf")).unwrap()
        ];

        let glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_fonts(fonts)
            .texture_filter_method(wgpu::FilterMode::Nearest)
            .build(&device, wgpu::TextureFormat::Bgra8Unorm);

        let mut init_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Hectic init CommandEncoder") });
        let texture = crate::graphics::load_packed(&device, &mut init_encoder);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            label: Some("Hectic Sampler"),
            ..Default::default()
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Float,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false, min_binding_size: None },
                        count: None,
                    }
                ],
                label: Some("Hectic BindGroupLayout"),
            });

        let window_size = window.inner_size();

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hectic uniform buffer"),
            contents: Uniforms::new(window_size.width, window_size.height).as_bytes(),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }
                }
            ],
            label: Some("Hectic BindGroup"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Hectic PipelineLayout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Hectic RenderPipeline"),
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor::default()),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8Unorm,
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::DstAlpha,
                    operation: wgpu::BlendOperation::Max,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float2],
                    },
                    wgpu::VertexBufferDescriptor {
                        stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![1 => Float2, 2 => Float2, 3 => Float, 4 => Float2, 5 => Float2, 6 => Float4, 7 => Int]
                    }
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
    
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
    
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        queue.submit(Some(init_encoder.finish()));

        let buffer_renderer = BufferRenderer {
            glyph_sections: Vec::new(),
            instances: Vec::new(),
            window_size: Vector2::new(window_size.width as f32, window_size.height as f32),
        };

        let instance_buffer = GpuBuffer::new(&device, 2560, "Hectic instances buffer");

        let square_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hectic square buffer"),
            contents: SQUARE.as_bytes(),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let renderer = Self {
            square_buffer, swap_chain, pipeline, window, device, queue, swap_chain_desc, surface,
            bind_group, uniform_buffer, instance_buffer, glyph_brush,
        };

        (renderer, buffer_renderer)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);

        let staging_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hectic uniform staging buffer"),
            contents: Uniforms::new(width, height).as_bytes(),
            usage: wgpu::BufferUsage::COPY_SRC,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Hectic CommandEncoder") });
        encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.uniform_buffer, 0, std::mem::size_of::<Uniforms>() as u64);
        self.queue.submit(Some(encoder.finish()));
    }

    pub fn render(&mut self, renderer: &mut BufferRenderer) {
        let offset = renderer.centering_offset() / 2.0;
        let dimensions = renderer.dimensions();

        if let Ok(frame) = self.swap_chain.get_current_frame() {
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Hectic CommandEncoder") });

            renderer.render_borders();

            self.instance_buffer.upload(&self.device, &mut encoder, &renderer.instances);

            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.output.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.5, g: 0.125, b: 0.125, a: 1.0 }),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                if self.instance_buffer.len > 0 {
                    rpass.set_pipeline(&self.pipeline);
                    rpass.set_bind_group(0, &self.bind_group, &[]);

                    rpass.set_vertex_buffer(0, self.square_buffer.slice(..));
                    let byte_len = self.instance_buffer.byte_len() as u64;
                    rpass.set_vertex_buffer(1, self.instance_buffer.buffer.slice(..byte_len));
                    rpass.draw(0 .. SQUARE.len() as u32, 0 .. self.instance_buffer.len as u32);
                }
            }

            let mut staging_belt = wgpu::util::StagingBelt::new(100);

            for section in renderer.glyph_sections.drain(..) {
                let layout = wgpu_glyph::PixelPositioner(section.layout);
                self.glyph_brush.queue_custom_layout(&section.to_borrowed(), &layout);
            }

            #[cfg(feature = "native")]
            self.glyph_brush.draw_queued_with_transform_and_scissoring(
                &self.device,
                &mut staging_belt,
                &mut encoder,
                &frame.output.view,
                wgpu_glyph::orthographic_projection(renderer.window_size.x as u32, renderer.window_size.y as u32),
                wgpu_glyph::Region { x: 0, y: offset.y as u32, width: renderer.window_size.x as u32, height: dimensions.y as u32 },
            ).unwrap();
            #[cfg(feature = "wasm")]
            self.glyph_brush.draw_queued(
                &self.device,
                &mut staging_belt,
                &mut encoder,
                &frame.output.view,
                self.swap_chain_desc.width,
                self.swap_chain_desc.height,
            ).unwrap();

            staging_belt.finish();

            self.queue.submit(Some(encoder.finish()));    
        }

        renderer.instances.clear();
    }

    pub fn request_redraw(&mut self) {
        self.window.request_redraw();
    }
}

struct GpuBuffer<'a, T> {
    buffer: wgpu::Buffer,
    capacity: usize,
    len: usize,
    label: &'a str,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: AsBytes> GpuBuffer<'a, T> {
    fn new(device: &wgpu::Device, base_capacity: usize, label: &'a str) -> Self {
        Self {
            capacity: base_capacity,
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: (base_capacity * std::mem::size_of::<T>()) as u64,
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
                mapped_at_creation: false,
            }),
            len: 0,
            label,
            _phantom: std::marker::PhantomData,
        }
    }

    fn upload(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, items: &[T]) {
        if items.is_empty() {
            self.len = 0;
            return;
        }

        if items.len() <= self.capacity {
            let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Gpu upload staging buffer"),
                contents: items.as_bytes(),
                usage: wgpu::BufferUsage::COPY_SRC,
            });
            encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.buffer, 0, items.as_bytes().len() as u64);
            self.len = items.len();
        } else {
            self.capacity = self.capacity * 2;
            log::debug!("Resizing buffer {} to {} items", self.label, self.capacity);
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label),
                size: (self.capacity * std::mem::size_of::<T>()) as u64,
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
                mapped_at_creation: true,
            });
            let byte_len = items.as_bytes().len() as u64;
            self.buffer.slice(..byte_len).get_mapped_range_mut().copy_from_slice(items.as_bytes());
            self.buffer.unmap();
            self.len = items.len();
        }
    }

    fn byte_len(&self) -> usize {
        self.len * std::mem::size_of::<T>()
    }
}

const SQUARE: [Vertex; 6] = [
    Vertex { point: [-1.0, -1.0] },
    Vertex { point: [ 1.0, -1.0] },
    Vertex { point: [-1.0,  1.0] },

    Vertex { point: [ 1.0, -1.0] },
    Vertex { point: [-1.0,  1.0] },
    Vertex { point: [ 1.0,  1.0] },
];

#[repr(C)]
#[derive(zerocopy::AsBytes, Clone, Debug)]
pub struct Vertex {
    point: [f32; 2],
}

#[repr(C)]
#[derive(zerocopy::AsBytes, Clone, Debug)]
pub struct Instance {
    center: [f32; 2],
    dimensions: [f32; 2],
    rotation: f32,
    uv_top_left: [f32; 2],
    uv_dimensions: [f32; 2],
    overlay: [f32; 4],
    overlay_only: i32
}

#[repr(C)]
#[derive(zerocopy::AsBytes, Clone, Debug)]
pub struct Uniforms {
    window_size: [f32; 2],
    virtual_size: [f32; 2]
}

impl Uniforms {
    fn new(width: u32, height: u32) -> Self {
        Self {
            window_size: [width as f32, height as f32],
            virtual_size: [WIDTH, HEIGHT]
        }
    }
}

pub struct BufferRenderer {
    window_size: Vector2<f32>,
    // We can't store a GlyphBrush directly here because on wasm the buffer
    // is a js type and thus not threadsafe.
    // todo: maybe store something lighter here so we can use cow strs
    glyph_sections: Vec<glyph_brush::OwnedSection<wgpu_glyph::Extra>>,
    instances: Vec<Instance>,
}

impl Default for BufferRenderer {
    fn default() -> Self {
        unreachable!()
    }
}

impl BufferRenderer {
    pub fn set_window_size(&mut self, width: u32, height: u32) {
        self.window_size = Vector2::new(width as f32, height as f32);
    }

    pub fn scale_factor(&self) -> f32 {
        (self.window_size.y / crate::HEIGHT)
            .min(self.window_size.x / crate::WIDTH)
    }

    fn centering_offset(&self) -> Vector2<f32> {
        self.window_size - self.dimensions()
    }

    fn dimensions(&self) -> Vector2<f32> {
        crate::DIMENSIONS * self.scale_factor()
    }

    pub fn render_sprite(&mut self, sprite: Image, pos: Vector2<f32>, rotation: f32, overlay: [f32; 4]) {
        self.render_sprite_with_dimensions(sprite, pos, sprite.size() * 2.0, rotation, overlay);
    }

    pub fn render_sprite_with_dimensions(&mut self, sprite: Image, center: Vector2<f32>, dimensions: Vector2<f32>, rotation: f32, overlay: [f32; 4]) {
        let (uv_x, uv_y, uv_w, uv_h) = sprite.coordinates();

        self.instances.push(Instance {
            center: center.into(),
            dimensions: dimensions.into(),
            rotation,
            uv_top_left: [uv_x, uv_y].into(),
            uv_dimensions: [uv_w, uv_h].into(),
            overlay,
            overlay_only: false as i32
        });
    }

    pub fn render_box(&mut self, center: Vector2<f32>, dimensions: Vector2<f32>, overlay: [f32; 4]) {
        self.instances.push(Instance {
            center: center.into(),
            dimensions: dimensions.into(),
            rotation: 0.0,
            uv_top_left: [0.0; 2],
            uv_dimensions: [0.0; 2],
            overlay,
            overlay_only: true as i32
        });
    }

    pub fn render_text(&mut self, text: &Text, mut pos: Vector2<f32>, color: [f32; 4]) {
        pos += self.centering_offset() / self.scale_factor() / 2.0;

        let scale = match text.font {
            0 => 160.0,
            1 => 24.0,
            _ => unreachable!()
        };

        let section = glyph_brush::OwnedSection {
            screen_position: (pos * self.scale_factor()).into(),
            layout: text.layout,
            text: vec![
                glyph_brush::OwnedText {
                    text: text.text.clone(),
                    scale: (scale * self.scale_factor()).into(),
                    font_id: wgpu_glyph::FontId(text.font),
                    extra: wgpu_glyph::Extra {
                        other: glyph_brush::Extra {
                            color,
                            ..Default::default()
                        },
                        draw_mode: wgpu_glyph::DrawMode::Pixelated(2.0 * self.scale_factor()),
                    }
                }
            ],
            ..Default::default()
        };

        self.glyph_sections.push(section);
    }

    pub fn render_circle(&mut self, center: Vector2<f32>, radius: f32) {
        for (x, y) in line_drawing::BresenhamCircle::new(center.x as i32, center.y as i32, (radius / 2.0) as i32) {
            let position = Vector2::new(x as f32, y as f32);
            let position = ((position - center) * 2.0) + center;

            self.render_box(position, Vector2::new(2.0, 2.0), [1.0; 4]);
        }
    }

    pub fn render_borders(&mut self) {
        let colour = [0.5, 0.125, 0.125, 1.0];
        let border_width = ((self.window_size.x / self.scale_factor()) - WIDTH) / 2.0;
        if border_width > 0.0 {
            self.render_box(Vector2::new(-border_width / 2.0, 0.0),        Vector2::new(border_width, HEIGHT * 2.0), colour);
            self.render_box(Vector2::new(WIDTH + border_width / 2.0, 0.0), Vector2::new(border_width, HEIGHT * 2.0), colour);
        }

        let border_height = ((self.window_size.y / self.scale_factor()) - HEIGHT) / 2.0;

        if border_height > 0.0 {
            self.render_box(Vector2::new(0.0, -border_height / 2.0), Vector2::new(WIDTH * 2.0, border_height), colour);
            self.render_box(Vector2::new(0.0, HEIGHT + border_height / 2.0), Vector2::new(WIDTH * 2.0, border_height), colour);
        }
    }
}
