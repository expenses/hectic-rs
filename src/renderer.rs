use winit::{
    event_loop::EventLoop,
    window::Window,
};

use cgmath::*;
use crate::{WIDTH, HEIGHT};
use crate::components::{Image, Text};
use zerocopy::*;

pub struct Renderer {
    swap_chain: wgpu::SwapChain,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Window,
    pipeline: wgpu::RenderPipeline,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    surface: wgpu::Surface,
    bind_group: wgpu::BindGroup,
    glyph_brush: wgpu_glyph::GlyphBrush<'static, ()>,
    square_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    instance_buffer: GpuBuffer<Instance>,
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

        let instance = wgpu::Instance::new();
        let surface = unsafe {
            instance.create_surface(&window)
        };

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: None,
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();
    
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        }, Some(&std::path::Path::new("trace"))).await.unwrap();

        let vs = include_bytes!("shader.vert.spv");
        let vs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());
    
        let fs = include_bytes!("shader.frag.spv");
        let fs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());
    
        let fonts: &[&[u8]] = &[
            include_bytes!("fonts/OldeEnglish.ttf"),
            include_bytes!("fonts/TinyUnicode.ttf")
        ];

        let glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_fonts_bytes(fonts)
            .unwrap()
            .texture_filter_method(wgpu::FilterMode::Nearest)
            .build(&device, wgpu::TextureFormat::Bgra8Unorm);

        let mut init_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Hectic init CommandEncoder") });
        let texture = crate::graphics::load_packed(&device, &mut init_encoder);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: wgpu::CompareFunction::Undefined,
            label: Some("Hectic Sampler")
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Float,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    }
                ],
                label: Some("Hectic BindGroupLayout"),
            });

        let window_size = window.inner_size();

        let uniform_buffer = device.create_buffer_with_data(
            Uniforms::new(window_size.width, window_size.height).as_bytes(),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(0 .. std::mem::size_of::<Uniforms>() as u64))
                }
            ],
            label: Some("Hectic BindGroup"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
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
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
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

        let instance_buffer = GpuBuffer::new(&device, 500);

        let renderer = Self {
            square_buffer: device.create_buffer_with_data(SQUARE.as_bytes(), wgpu::BufferUsage::VERTEX),
            swap_chain, pipeline, window, device, queue, swap_chain_desc, surface, bind_group, glyph_brush,
            uniform_buffer, instance_buffer
        };

        (renderer, buffer_renderer)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);

        let staging_buffer = self.device.create_buffer_with_data(Uniforms::new(width, height).as_bytes(), wgpu::BufferUsage::COPY_SRC);
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Hectic CommandEncoder") });
        encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.uniform_buffer, 0, std::mem::size_of::<Uniforms>() as u64);
        self.queue.submit(Some(encoder.finish()));
    }

    pub fn render(&mut self, renderer: &mut BufferRenderer) {
        let offset = renderer.centering_offset() / 2.0;
        let dimensions = renderer.dimensions();

        if let Ok(frame) = self.swap_chain.get_next_texture() {
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Hectic CommandEncoder") });

            self.instance_buffer.upload(&self.device, &mut encoder, &renderer.instances);

            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color { r: 0.5, g: 0.125, b: 0.125, a: 1.0 },
                    }],
                    depth_stencil_attachment: None,
                });

                if self.instance_buffer.len > 0 {
                    #[cfg(feature = "native")]
                    rpass.set_scissor_rect(offset.x as u32, offset.y as u32, dimensions.x as u32, dimensions.y as u32);

                    rpass.set_pipeline(&self.pipeline);
                    rpass.set_bind_group(0, &self.bind_group, &[]);

                    rpass.set_vertex_buffer(0, self.square_buffer.slice(..));
                    let byte_len = self.instance_buffer.byte_len() as u64;
                    rpass.set_vertex_buffer(1, self.instance_buffer.buffer.slice(..byte_len));
                    rpass.draw(0 .. SQUARE.len() as u32, 0 .. self.instance_buffer.len as u32);
                }
            }

            for section in renderer.glyph_sections.drain(..) {
                let layout = wgpu_glyph::PixelPositioner(section.layout);
                self.glyph_brush.queue_custom_layout(&section, &layout);
            }

            fn orthographic_projection(width: f32, height: f32) -> [f32; 16] {
                [
                    2.0 / width, 0.0, 0.0, 0.0,
                    0.0, -2.0 / height, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    -1.0, 1.0, 0.0, 1.0,
                ]
            }

            #[cfg(feature = "native")]
            self.glyph_brush.draw_queued_with_transform_and_scissoring(
                &self.device,
                &mut encoder,
                &frame.view,
                orthographic_projection(renderer.window_size.x, renderer.window_size.y),
                wgpu_glyph::Region { x: offset.x as u32, y: offset.y as u32, width: dimensions.x as u32, height: dimensions.y as u32 },
            ).unwrap();
            #[cfg(feature = "wasm")]
            self.glyph_brush.draw_queued(
                &self.device,
                &mut encoder,
                &frame.view,
                self.swap_chain_desc.width,
                self.swap_chain_desc.height,
            ).unwrap();

            self.queue.submit(Some(encoder.finish()));    
        }

        renderer.instances.clear();
    }

    pub fn request_redraw(&mut self) {
        self.window.request_redraw();
    }
}

struct GpuBuffer<T> {
    buffer: wgpu::Buffer,
    capacity: usize,
    len: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: AsBytes> GpuBuffer<T> {
    fn new(device: &wgpu::Device, base_capacity: usize) -> Self {
        Self {
            capacity: base_capacity,
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (base_capacity * std::mem::size_of::<T>()) as u64,
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            }),
            len: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    fn upload(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, items: &[T]) {
        if items.is_empty() {
            self.len = 0;
            return;
        }

        if items.len() <= self.capacity {
            let staging_buffer = device.create_buffer_with_data(items.as_bytes(), wgpu::BufferUsage::COPY_SRC);
            encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.buffer, 0, items.as_bytes().len() as u64);
            self.len = items.len();
        } else {
            self.capacity = self.capacity * 2;
            println!("resizing to {} items", self.capacity);
            let mut buffer = device.create_buffer_mapped(&wgpu::BufferDescriptor {
                label: None,
                size: (self.capacity * std::mem::size_of::<T>()) as u64,
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            });
            let byte_len = items.as_bytes().len();
            buffer.data()[..byte_len].copy_from_slice(items.as_bytes());
            self.buffer = buffer.finish();
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
    glyph_sections: Vec<wgpu_glyph::OwnedVariedSection<wgpu_glyph::DrawMode>>,
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

        let section = wgpu_glyph::OwnedVariedSection {
            screen_position: (pos * self.scale_factor()).into(),
            layout: text.layout,
            text: vec![
                wgpu_glyph::OwnedSectionText {
                    text: text.text.clone(),
                    scale: wgpu_glyph::Scale::uniform(scale * self.scale_factor()),
                    color,
                    font_id: wgpu_glyph::FontId(text.font),
                    custom: wgpu_glyph::DrawMode::pixelated(2.0 * self.scale_factor()),
                }
            ],
            ..wgpu_glyph::OwnedVariedSection::default()
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
}
