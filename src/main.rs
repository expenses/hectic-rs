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
    #[cfg(feature = "wasm")]
    wasm_bindgen_futures::spawn_local(run());
    #[cfg(feature = "native")]
    futures::executor::block_on(run());
}

async fn run() {
    #[cfg(feature = "wasm")]
    {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Trace).unwrap();
    }
    #[cfg(feature = "native")]
    env_logger::init();

    let event_loop = EventLoop::new();

    let (mut renderer, buffer_renderer) = renderer::Renderer::new(&event_loop).await;

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
    world.register::<components::Cooldown>();
    world.register::<components::Friendly>();
    world.register::<components::Enemy>();
    world.register::<components::Hitbox>();
    world.register::<components::Health>();
    world.register::<components::Explosion>();
    world.register::<components::Invulnerability>();
    world.register::<components::Text>();
    world.register::<components::TargetPlayer>();

    world.insert(resources::KeyPresses(vec![]));
    world.insert(resources::KeyboardState::default());
    world.insert(buffer_renderer);
    world.insert(resources::GameTime(0.0));
    world.insert(resources::BulletSpawner::default());
    world.insert(resources::DamageTracker::default());
    world.insert(resources::PlayerPositions::default());

    stages::stage_one(&mut world);
    stages::stage_two(&mut world);
    
    let db = DispatcherBuilder::new()
        .with(systems::KillOffscreen, "KillOffscreen", &[])
        .with(systems::MoveEntities, "MoveEntities", &[])
        .with(systems::HandleKeypresses, "HandleKeypresses", &[])
        .with(systems::Control, "Control", &[])
        .with(systems::SetPlayerPositions, "SetPlayerPositions", &[])
        .with(systems::FireBullets, "FireBullets", &[])
        .with(systems::SpawnBullets, "SpawnBullets", &[])
        .with(systems::RepeatBackgroundLayers, "RepeatBackgroundLayers", &[])
        .with(systems::TickTime, "TickTime", &[])
        .with(systems::StartTowardsPlayer, "StartTowardsPlayer", &["TickTime"])
        .with(systems::AddOnscreen, "AddOnscreen", &[])
        .with(systems::Collisions, "Collisions", &[])
        .with(systems::ApplyCollisions, "ApplyCollisions", &["Collisions"])
        .with(systems::ExplosionImages, "ExplosionImages", &["ApplyCollisions"])
        .with(systems::RenderSprite, "RenderSprite", &["MoveEntities", "Control", "SpawnBullets", "ExplosionImages"])
        .with(systems::RenderText, "RenderText", &["RenderSprite"]);
        //.with(systems::RenderHitboxes, "RenderHitboxes", &["RenderSprite"]);

    log::debug!("{:?}", db);

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
