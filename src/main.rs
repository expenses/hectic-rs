use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

use specs::prelude::*;
use cgmath::Vector2;
use zerocopy::AsBytes;

mod graphics;
mod components;
mod resources;
mod systems;
mod stages;

struct Renderer2 {
    swap_chain: wgpu::SwapChain,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Window,
    pipeline: wgpu::RenderPipeline,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    surface: wgpu::Surface,
    bind_group: wgpu::BindGroup,
}

impl Renderer2 {
    async fn new(event_loop: &EventLoop<()>) -> (Self, graphics::Resources) {
        let window = WindowBuilder::new()
            .with_inner_size(winit::dpi::PhysicalSize { width: 480.0, height: 640.0 })
            .with_resizable(false)
            .build(event_loop)
            .unwrap();
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
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
    

        let mut init_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        let resources = graphics::Resources::load(&device, &mut init_encoder);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
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
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                ],
            });
        let bind_group = build_bind_group(&device, &bind_group_layout, &resources.sprites, &sampler);

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
                ],
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
    
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
    
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        queue.submit(&[init_encoder.finish()]);

        let renderer = Self {
            swap_chain, pipeline, window, device, queue, swap_chain_desc, surface, bind_group,
        };

        (renderer, resources)
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    fn render(&mut self, renderer: &mut resources::Renderer) {
        
        let v = self.device.create_buffer_with_data(renderer.vertices.as_bytes(), wgpu::BufferUsage::VERTEX);
        let i = self.device.create_buffer_with_data(renderer.indices.as_bytes(), wgpu::BufferUsage::INDEX);


        let output = self.swap_chain.get_next_texture().unwrap();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
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
        self.queue.submit(&[encoder.finish()]);

        renderer.vertices.clear();
        renderer.indices.clear();
    }
}

fn build_bind_group(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, sprite: &graphics::Sprite, sampler: &wgpu::Sampler) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        bindings: &[
            wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&sprite.texture_view),
            },
            wgpu::Binding {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn main() {
    futures::executor::block_on(async {
        let event_loop = EventLoop::new();

        let (mut renderer, _) = Renderer2::new(&event_loop).await;

        let mut world = World::new();
        world.register::<components::Position>();
        world.register::<components::Image>();
        world.register::<components::Movement>();
        world.register::<components::DieOffscreen>();
        world.register::<components::BackgroundLayer>();
        world.register::<components::Controllable>();
        world.register::<components::FrozenUntil>();
        world.register::<components::BeenOnscreen>();

        world.insert(resources::KeyPresses(vec![]));
        world.insert(resources::Controls::default());
        let size = renderer.window.inner_size();
        world.insert(resources::Renderer {
            vertices: Vec::new(),
            indices: Vec::new(),
            dpi_factor: renderer.window.scale_factor() as f32,
            window_size: (size.width as f32, size.height as f32)
        });
        world.insert(resources::GameTime(0.0));

        stages::stage_one(&mut world);
        
        let db = DispatcherBuilder::new()
            .with(systems::KillOffscreen, "kill", &[])
            .with(systems::MoveEntities, "mov", &[])
            .with(systems::HandleKeypresses, "key", &[])
            .with(systems::Control, "ctrl", &[])
            .with(systems::RenderSprite, "rs", &["mov", "ctrl"])
            .with(systems::RepeatBackgroundLayers, "rbl", &[])
            .with(systems::TickTime, "tick", &[])
            .with(systems::AddOnscreen, "add_on", &[]);

        println!("{:?}", db);

        let mut dispatcher = db.build();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    renderer.resize(size.width as u32, size.height as u32);
                    world.fetch_mut::<resources::Renderer>().window_size = (size.width as f32, size.height as f32);
                    *control_flow = ControlFlow::Poll;
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(code),
                            state,
                            ..
                        },
                    ..
                } => {
                    let pressed = state == ElementState::Pressed;
    
                    match code {
                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        _ => world.fetch_mut::<resources::KeyPresses>().0.push((code, pressed)),
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                dispatcher.dispatch(&world);
                world.maintain();
                renderer.window.request_redraw()
            },
            Event::RedrawRequested(_) => renderer.render(&mut world.fetch_mut()),
            _ => {}
        });
    })
}

#[repr(C)]
#[derive(zerocopy::AsBytes, Clone, Debug)]
pub struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2]
}

const WIDTH: f32 = 480.0;
const HEIGHT: f32 = 640.0;
