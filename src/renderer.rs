use winit::{
    event_loop::EventLoop,
    window::{WindowBuilder, Window},
};

use cgmath::*;
use crate::components::Image;
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
}

impl Renderer {
    pub async fn new(event_loop: &EventLoop<()>) -> (Self, BufferRenderer) {
        let window = WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize { width: 480.0, height: 640.0 })
            .build(event_loop)
            .unwrap();

        let size = window.inner_size();
        // Non-integer dpi_factors (such as 1.3333334) on my laptop don't render the pixel art very well,
        // so we floor the dpi factor and use that for the window size.
        // TODO: fix by allowing the window to be resized with a nice fullscreen mode.
        let dpi_factor = window.scale_factor().floor() as f32;
        window.set_inner_size(winit::dpi::PhysicalSize { width: 480.0 * dpi_factor, height: 640.0 * dpi_factor });

        let surface = wgpu::Surface::create(&window);

        let adapter = wgpu::Adapter::request(
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
        }).await;

        let vs = include_bytes!("shader.vert.spv");
        let vs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());
    
        let fs = include_bytes!("shader.frag.spv");
        let fs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());
    
        let font: &[u8] = include_bytes!("resources/fonts/OldeEnglish.ttf");

        let mut glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_font_bytes(font)
            .unwrap()
            .texture_filter_method(wgpu::FilterMode::Nearest)
            .build(&device, wgpu::TextureFormat::Bgra8UnormSrgb);

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
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: blend_descriptor,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
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
            alpha_to_coverage_enabled: false,
        });
    
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
    
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        queue.submit(&[init_encoder.finish()]);

        let buffer_renderer = BufferRenderer {
            vertices: Vec::new(),
            indices: Vec::new(),
            window_size: Vector2::new(size.width as f32, size.height as f32),
            glyph_brush, dpi_factor,
        };

        let renderer = Self {
            swap_chain, pipeline, window, device, queue, swap_chain_desc, surface, bind_group
        };

        (renderer, buffer_renderer)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    pub fn render(&mut self, renderer: &mut BufferRenderer) {
        let v = self.device.create_buffer_with_data(renderer.vertices.as_bytes(), wgpu::BufferUsage::VERTEX);
        let i = self.device.create_buffer_with_data(renderer.indices.as_bytes(), wgpu::BufferUsage::INDEX);

        let output = self.swap_chain.get_next_texture().unwrap();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &output.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);

            rpass.set_index_buffer(&i, 0, 0);
            rpass.set_vertex_buffer(0, &v, 0, 0);
            rpass.draw_indexed(0 .. renderer.indices.len() as u32, 0, 0 .. 1);
        }

        for section in renderer.glyph_sections.drain(..) {
            self.glyph_brush.queue(section);
        }
        self.glyph_brush.draw_queued(
            &self.device,
            &mut encoder,
            &output.view,
            self.swap_chain_desc.width,
            self.swap_chain_desc.height,
        ).unwrap();

        self.queue.submit(&[encoder.finish()]);

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
    glyph_sections: Vec<wgpu_glyph::Section<'static, wgpu_glyph::DrawMode>>,
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

    pub fn render_sprite(&mut self, sprite: Image, pos: Vector2<f32>, overlay: [f32; 4]) {
        let len = self.vertices.len() as i16;
        let (pos_x, pos_y, width, height) = sprite.coordinates();

        // dpi factor?
        let pos = pos * 2.0 * self.scale_factor();
        let pos = pos - self.window_size;
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
        let pos = pos - self.window_size;
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

    pub fn render_text(&mut self, text: &'static str, pos: Vector2<f32>, font: usize) {
        let scale = match font {
            0 => 160.0,
            1 => 32.0,
            _ => unreachable!()
        };

        let section = wgpu_glyph::Section {
            text,
            screen_position: (pos * self.scale_factor()).into(),
            scale: wgpu_glyph::Scale::uniform(scale * self.scale_factor()),
            color: [1.0; 4],
            layout: wgpu_glyph::Layout::default().h_align(wgpu_glyph::HorizontalAlign::Center),
            custom: wgpu_glyph::DrawMode::pixelated(2.0 * self.scale_factor()),
            
            font_id: wgpu_glyph::FontId(font),
            ..wgpu_glyph::Section::default()
        };

        self.glyph_sections.push(section);
    }
}
