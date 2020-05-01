use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use cgmath::Vector2;
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
    world.register::<components::Player>();
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
    world.register::<components::PowerOrb>();
    world.register::<components::PowerBar>();
    world.register::<components::Circle>();
    world.register::<components::CollidesWithBomb>();

    world.insert(resources::ControlsState::load());
    world.insert(buffer_renderer);
    world.insert(resources::GameTime::default());
    world.insert(resources::PlayerPositions::default());
    world.insert(resources::Mode::default());

    let db = DispatcherBuilder::new()
        .with(systems::ExplosionImages, "ExplosionImages", &[])
        .with(systems::TogglePaused, "TogglePaused", &[])
        .with(systems::KillOffscreen, "KillOffscreen", &[])
        .with(systems::ExpandBombs, "ExpandCircles", &[])
        .with(systems::MoveEntities, "MoveEntities", &[])
        .with(systems::CollectOrbs, "CollectOrbs", &[])
        .with(systems::Control, "Control", &[])
        .with(systems::SetPlayerPositions, "SetPlayerPositions", &[])
        .with(systems::FireBullets, "FireBullets", &[])
        .with(systems::RepeatBackgroundLayers, "RepeatBackgroundLayers", &[])
        .with(systems::TickTime, "TickTime", &[])
        .with(systems::StartTowardsPlayer, "StartTowardsPlayer", &["TickTime"])
        .with(systems::AddOnscreen, "AddOnscreen", &[])
        .with(systems::Collisions, "Collisions", &[])
        .with(systems::RenderSprite::default(), "RenderSprite", &["MoveEntities", "Control", "ExplosionImages"])
        .with(systems::RenderText, "RenderText", &["RenderSprite"])
        .with(systems::RenderBombs, "RenderBombs", &["RenderSprite"])
        .with(systems::RenderHitboxes, "RenderHitboxes", &["RenderSprite"])
        .with(systems::RenderUI, "RenderUI", &["RenderSprite"]);

    log::debug!("{:?}", db);

    let mut playing_dispatcher = db.build();

    let mut paused_dispatcher = DispatcherBuilder::new()
        .with(systems::TogglePaused, "TogglePaused", &[])
        .with(systems::ControlMenu, "ControlMenu", &[])
        .with(systems::RenderSprite::default(), "RenderSprite", &[])
        .with(systems::RenderText, "RenderText", &["RenderSprite"])
        .with(systems::RenderBombs, "RenderBombs", &["RenderSprite"])
        .with(systems::RenderHitboxes, "RenderHitboxes", &["RenderSprite"])
        .with(systems::RenderUI, "RenderUI", &["RenderSprite"])
        .with(systems::RenderPauseBackground, "RenderPauseBackground", &["RenderSprite"])
        .with(systems::RenderMenu, "RenderMenu", &["RenderSprite"])
        .build();

    let mut menu_dispatcher = DispatcherBuilder::new()
        .with(systems::ControlMenu, "ControlMenu", &[])
        .with(systems::RenderMenu, "RenderMenu", &[])
        .build();

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
                    _ => world.fetch_mut::<resources::ControlsState>().press(code, pressed),
                }
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            let mode: resources::Mode = *world.fetch();
            match mode {
                resources::Mode::MainMenu { .. } | resources::Mode::Stages { .. } | resources::Mode::Controls { .. } => menu_dispatcher.dispatch(&world),
                resources::Mode::Playing => playing_dispatcher.dispatch(&world),
                resources::Mode::Paused { .. } => paused_dispatcher.dispatch(&world),
                resources::Mode::Quit => *control_flow = ControlFlow::Exit,
                resources::Mode::StageOne { multiplayer } => {
                    stages::stage_one(&mut world, multiplayer);
                    *world.fetch_mut() = resources::Mode::Playing;
                },
                resources::Mode::StageTwo { multiplayer } => {
                    stages::stage_two(&mut world, multiplayer);
                    *world.fetch_mut() = resources::Mode::Playing;
                }
            }
            world.maintain();
            renderer.request_redraw();
        },
        Event::RedrawRequested(_) => renderer.render(&mut world.fetch_mut()),
        Event::LoopDestroyed => world.fetch::<resources::ControlsState>().save(),
        _ => {}
    });
}

const WIDTH: f32 = 480.0;
const HEIGHT: f32 = 640.0;
const DIMENSIONS: Vector2<f32> = Vector2::new(WIDTH, HEIGHT);
const ZERO: Vector2<f32> = Vector2::new(0.0, 0.0);
const MIDDLE: Vector2<f32> = Vector2::new(WIDTH / 2.0, HEIGHT / 2.0);
