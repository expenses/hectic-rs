use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use specs::prelude::*;

mod graphics;
mod components;
mod resources;
mod systems;
mod stages;
mod renderer;

use std::alloc::System;

#[global_allocator]
static GLOBAL: System = System;


fn main() {
    let event_loop = EventLoop::new();

    let (mut renderer, buffer_renderer) = futures::executor::block_on(renderer::Renderer::new(&event_loop));

    let mut world = World::new();
    world.register::<components::Position>();
    world.register::<components::Image>();
    world.register::<components::Movement>();
    world.register::<components::DieOffscreen>();
    world.register::<components::BackgroundLayer>();
    world.register::<components::Controllable>();
    world.register::<components::FrozenUntil>();
    world.register::<components::BeenOnscreen>();
    world.register::<components::FiresBullets>();

    world.insert(resources::KeyPresses(vec![]));
    world.insert(resources::KeyboardState::default());
    world.insert(buffer_renderer);
    world.insert(resources::GameTime(0.0));
    world.insert(resources::BulletSpawner::default());

    stages::stage_one(&mut world);
    
    let db = DispatcherBuilder::new()
        .with(systems::KillOffscreen, "KillOffscreen", &[])
        .with(systems::MoveEntities, "MoveEntities", &[])
        .with(systems::HandleKeypresses, "HandleKeypresses", &[])
        .with(systems::Control, "Control", &[])
        .with(systems::FireBullets, "FireBullets", &[])
        .with(systems::SpawnBullets, "SpawnBullets", &[])
        .with(systems::RenderSprite, "RenderSprite", &["MoveEntities", "Control", "SpawnBullets"])
        .with(systems::RepeatBackgroundLayers, "RepeatBackgroundLayers", &[])
        .with(systems::TickTime, "TickTime", &[])
        .with(systems::AddOnscreen, "AddOnscreen", &[]);

    println!("{:?}", db);

    let mut dispatcher = db.build();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::Resized(size) => {
                renderer.resize(size.width as u32, size.height as u32);
                world.fetch_mut::<renderer::BufferRenderer>().set_window_size(size.width, size.height);
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
            renderer.request_redraw();
        },
        Event::RedrawRequested(_) => renderer.render(&mut world.fetch_mut()),
        _ => {}
    });
}

const WIDTH: f32 = 480.0;
const HEIGHT: f32 = 640.0;
