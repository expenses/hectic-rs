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
    bind_group_layout: wgpu::BindGroupLayout,
    texture: wgpu::TextureView,
    sampler: wgpu::Sampler,
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
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: Some(&surface),
        }).await.unwrap();
    
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            shader_validation: true,
        }, Some(&std::path::Path::new("trace"))).await.unwrap();

        let vs_module =
            device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
    
        let fs_module =
            device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));
    
        let fonts = vec![
            wgpu_glyph::ab_glyph::FontRef::try_from_slice(include_bytes!("fonts/OldeEnglish.ttf")).unwrap(),
            wgpu_glyph::ab_glyph::FontRef::try_from_slice(include_bytes!("fonts/TinyUnicode.ttf")).unwrap(),
        ];

        let glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_fonts(fonts)
            .initial_cache_size((512, 512))
            .texture_filter_method(wgpu::FilterMode::Nearest)
            .build(&device, wgpu::TextureFormat::Bgra8Unorm);

        let mut init_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Hectic init CommandEncoder".into()) });
        let texture = crate::graphics::load_packed(&device, &mut init_encoder);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Hectic Sampler".into()),
            .. Default::default()
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
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("Hectic BindGroupLayout".into()),
            });

        let window_size = window.inner_size();

        let bind_group = create_bind_group(&device, &bind_group_layout, &texture, &sampler, Uniforms::new(window_size.width, window_size.height));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: Default::default(),
            label: Some("Hectic PipelineLayout".into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Hectic Pipeline".into()),
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main".into(),
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main".into(),
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
                        attributes: &wgpu::vertex_attr_array![1 => Float2, 2 => Float2, 3 => Float, 4 => Float2, 5 => Float2, 6 => Float4, 7 => Int],
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

        let square_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hectic square buffer"),
            contents: SQUARE.as_bytes(),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let renderer = Self {
            square_buffer, swap_chain, pipeline, window, device, queue, swap_chain_desc, surface,
            bind_group, glyph_brush, bind_group_layout, texture, sampler,
        };

        (renderer, buffer_renderer)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
        self.bind_group = create_bind_group(&self.device, &self.bind_group_layout, &self.texture, &self.sampler, Uniforms::new(width, height));
    }

    pub fn render(&mut self, renderer: &mut BufferRenderer) {        
        let buffers = if !renderer.instances.is_empty() {
            Some(
                self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Hectic instances buffer"),
                    contents: renderer.instances.as_bytes(),
                    usage: wgpu::BufferUsage::VERTEX,
                })
            )
        } else {
            None
        };

        let offset = renderer.centering_offset() / 2.0;
        let dimensions = renderer.dimensions();

        if let Ok(frame) = self.swap_chain.get_current_frame() {
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Hectic CommandEncoder".into())
            });

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
    
                if let Some(instances) = &buffers {
                    #[cfg(feature = "native")]
                    rpass.set_scissor_rect(offset.x as u32, offset.y as u32, dimensions.x as u32, dimensions.y as u32);
    
                    rpass.set_pipeline(&self.pipeline);
                    rpass.set_bind_group(0, &self.bind_group, &[]);
    
                    rpass.set_vertex_buffer(0, self.square_buffer.slice(..));
                    rpass.set_vertex_buffer(1, instances.slice(..));
                    rpass.draw(0 .. SQUARE.len() as u32, 0 .. renderer.instances.len() as u32);
                }
            }
    
            for section in renderer.glyph_sections.drain(..) {
                let layout = wgpu_glyph::PixelPositioner(section.layout);
                let section = section.to_borrowed();
                self.glyph_brush.queue_custom_layout(section, &layout);
            }

            #[cfg(feature = "native")]
            self.glyph_brush.draw_queued_with_transform_and_scissoring(
                &self.device,
                &mut wgpu::util::StagingBelt::new(100),
                &mut encoder,
                &frame.output.view,
                wgpu_glyph::orthographic_projection(renderer.window_size.x as u32, renderer.window_size.y as u32),
                wgpu_glyph::Region { x: offset.x as u32, y: offset.y as u32, width: dimensions.x as u32, height: dimensions.y as u32 },
            ).unwrap();
            #[cfg(feature = "wasm")]
            self.glyph_brush.draw_queued(
                &self.device,
                &mut wgpu::util::StagingBelt::new(100),
                &mut encoder,
                &frame.output.view,
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

fn create_bind_group(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, texture: &wgpu::TextureView, sampler: &wgpu::Sampler, uniforms: Uniforms) -> wgpu::BindGroup {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Hectic uniforms"),
        contents: uniforms.as_bytes(),
        usage: wgpu::BufferUsage::UNIFORM
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(buffer.slice(..)),
            }
        ],
        label: Some("Hectic BindGroup".into()),
    })
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
                    scale: glyph_brush::ab_glyph::PxScale::from(scale * self.scale_factor()),
                    font_id: wgpu_glyph::FontId(text.font),
                    extra: wgpu_glyph::Extra {
                        draw_mode: wgpu_glyph::DrawMode::Pixelated(2.0 * self.scale_factor()),
                        other: glyph_brush::Extra {
                            color,
                            ..Default::default()
                        },
                    },
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
}
