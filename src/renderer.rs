use winit::{
    event_loop::EventLoop,
    window::{WindowBuilder, Window},
};

use cgmath::*;
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
    // Need to hold the adapter for https://bugzilla.mozilla.org/show_bug.cgi?id=1634239
    _adapter: wgpu::Adapter,
}

impl Renderer {
    pub async fn new(event_loop: &EventLoop<()>) -> (Self, BufferRenderer) {
        let window = WindowBuilder::new()
            //.with_inner_size(winit::dpi::LogicalSize { width: 480.0, height: 640.0 })
            // Debug only
            //.with_resizable(false)
            .build(event_loop)
            .unwrap();

        let size = window.inner_size();
        // Non-integer dpi_factors (such as 1.3333334) on my laptop don't render the pixel art very well,
        // so we floor the dpi factor and use that for the window size.
        let dpi_factor = window.scale_factor().floor() as f32;
        window.set_min_inner_size(Some(winit::dpi::PhysicalSize { width: 480.0 * dpi_factor, height: 640.0 * dpi_factor }));

        #[cfg(feature = "wasm")]
        {
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
        }).await.unwrap();

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

        let mut init_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let packed_texture = crate::graphics::load_packed(&device, &mut init_encoder);
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
                ],
                label: None,
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&packed_texture),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let blend_descriptor = wgpu::BlendDescriptor {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        };
    
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
                color_blend: blend_descriptor.clone(),
                alpha_blend: blend_descriptor,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float4,
                            offset: 16,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: true,
        });
    
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
    
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        queue.submit(Some(init_encoder.finish()));

        let buffer_renderer = BufferRenderer {
            vertices: Vec::new(),
            indices: Vec::new(),
            glyph_sections: Vec::new(),
            window_size: Vector2::new(size.width as f32, size.height as f32),
        };

        let renderer = Self {
            swap_chain, pipeline, window, device, queue, swap_chain_desc, surface, bind_group, glyph_brush, _adapter: adapter,
        };

        (renderer, buffer_renderer)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    pub fn render(&mut self, renderer: &mut BufferRenderer) {
        let buffers = if !renderer.vertices.is_empty() {
            Some((
                self.device.create_buffer_with_data(renderer.vertices.as_bytes(), wgpu::BufferUsage::VERTEX),
                self.device.create_buffer_with_data(renderer.indices.as_bytes(), wgpu::BufferUsage::INDEX)
            ))
        } else {
            None
        };

        let output = self.swap_chain.get_next_texture().unwrap();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &output.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color { r: 0.5, g: 0.125, b: 0.125, a: 1.0 },
                }],
                depth_stencil_attachment: None,
            });

            if let Some((vertices, indices)) = &buffers {
                rpass.set_pipeline(&self.pipeline);
                rpass.set_bind_group(0, &self.bind_group, &[]);

                rpass.set_index_buffer(indices, 0, 0);
                rpass.set_vertex_buffer(0, vertices, 0, 0);
                rpass.draw_indexed(0 .. renderer.indices.len() as u32, 0, 0 .. 1);
            }
        }

        for section in renderer.glyph_sections.drain(..) {
            let layout = wgpu_glyph::PixelPositioner(section.layout);
            self.glyph_brush.queue_custom_layout(&section, &layout);
        }
        self.glyph_brush.draw_queued(
            &self.device,
            &mut encoder,
            &output.view,
            self.swap_chain_desc.width,
            self.swap_chain_desc.height,
        ).unwrap();

        self.queue.submit(Some(encoder.finish()));

        renderer.vertices.clear();
        renderer.indices.clear();
    }

    pub fn request_redraw(&mut self) {
        self.window.request_redraw();
    }
}

#[repr(C)]
#[derive(zerocopy::AsBytes, Clone, Debug)]
pub struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
    overlay: [f32; 4],
}

pub struct BufferRenderer {
    vertices: Vec<Vertex>,
    indices: Vec<i16>,
    window_size: Vector2<f32>,
    // We can't store a GlyphBrush directly here because on wasm the buffer
    // is a js type and thus not threadsafe.
    // todo: maybe store something lighter here so we can use cow strs
    glyph_sections: Vec<wgpu_glyph::OwnedVariedSection<wgpu_glyph::DrawMode>>,
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

    fn centering_x_offset(&self) -> f32 {
        self.window_size.x - crate::WIDTH * self.scale_factor()
    }

    pub fn render_sprite(&mut self, sprite: Image, pos: Vector2<f32>, overlay: [f32; 4]) {
        let len = self.vertices.len() as i16;
        let (pos_x, pos_y, width, height) = sprite.coordinates();

        // dpi factor?
        let pos = pos * 2.0 * self.scale_factor();
        let mut pos = pos - self.window_size;
        pos.x += self.centering_x_offset();
        let pos = pos.div_element_wise(self.window_size);

        let sprite_size = (sprite.size() * 2.0)
            .div_element_wise(self.window_size)
            * self.scale_factor();
        
        let x = pos.x;
        let y = -pos.y;
        let s_w = sprite_size.x;
        let s_h = -sprite_size.y;

        self.vertices.extend_from_slice(&[
            Vertex{pos: [x + s_w, y - s_h], uv: [pos_x + width, pos_y], overlay},
            Vertex{pos: [x - s_w, y - s_h], uv: [pos_x, pos_y], overlay},
            Vertex{pos: [x - s_w, y + s_h], uv: [pos_x, pos_y + height], overlay},
            Vertex{pos: [x + s_w, y + s_h], uv: [pos_x + width, pos_y + height], overlay},
        ]);

        self.indices.extend_from_slice(&[len, len + 1, len + 2, len + 2, len + 3, len]);
    }

    pub fn render_box(&mut self, pos: Vector2<f32>, mut size: Vector2<f32>) {
        let len = self.vertices.len() as i16;

        size.x = size.x.max(2.0);
        size.y = size.y.max(2.0);

        // dpi factor?
        let pos = pos * 2.0 * self.scale_factor();
        let mut pos = pos - self.window_size;
        pos.x += self.centering_x_offset();
        let pos = pos.div_element_wise(self.window_size);

        let sprite_size = size
            .div_element_wise(self.window_size)
            * self.scale_factor();
        
        let x = pos.x;
        let y = -pos.y;
        let s_w = sprite_size.x;
        let s_h = -sprite_size.y;

        let overlay = [1.0, 0.0, 0.0, 1.0];

        self.vertices.extend_from_slice(&[
            Vertex{pos: [x + s_w, y - s_h], uv: [0.0; 2], overlay},
            Vertex{pos: [x - s_w, y - s_h], uv: [0.0; 2], overlay},
            Vertex{pos: [x - s_w, y + s_h], uv: [0.0; 2], overlay},
            Vertex{pos: [x + s_w, y + s_h], uv: [0.0; 2], overlay},
        ]);

        self.indices.extend_from_slice(&[len, len + 1, len + 2, len + 2, len + 3, len]);
    }

    pub fn render_text(&mut self, text: &Text, mut pos: Vector2<f32>) {
        pos.x += self.centering_x_offset() / self.scale_factor() / 2.0;

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
                    color: [1.0; 4],
                    font_id: wgpu_glyph::FontId(text.font),
                    custom: wgpu_glyph::DrawMode::pixelated(2.0 * self.scale_factor()),
                }
            ],
            ..wgpu_glyph::OwnedVariedSection::default()
        };

        self.glyph_sections.push(section);
    }
}
