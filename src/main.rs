use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use structopt::StructOpt;
use specs::prelude::*;

mod graphics;
mod components;
mod resources;
mod systems;
mod stages;
mod renderer;
mod networking;

use std::alloc::System;

#[global_allocator]
static GLOBAL: System = System;

#[derive(structopt::StructOpt)]
enum Options {
    Singleplayer,
    MultiplayerServer,
    MultiplayerClient {
        address: std::net::SocketAddr 
    }
}

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

    let options = Options::from_args();

    let mut world = setup_world();
    let mut db = DispatcherBuilder::new();

    match options {
        Options::Singleplayer => {
            let event_loop = EventLoop::new();
            let (mut renderer, buffer_renderer) = renderer::Renderer::new(&event_loop).await;

            world.insert(buffer_renderer);

            stages::stage_one(&mut world);
            

            let db = setup_game_systems(db);
            let db = setup_rendering_systems(db, &["MoveEntities", "Control", "SpawnBullets", "ExplosionImages"]);

            println!("{:?}", db);

            let mut dispatcher = db.build();

            run_event_loop(event_loop, renderer, world, dispatcher);
        },
        Options::MultiplayerServer => {
            stages::stage_one(&mut world);
            let db = setup_game_systems(db);
            println!("{:?}", db);
            let mut dispatcher = db.build();

            let mut server = networking::Server::new(world, dispatcher);

            loop {
                std::thread::sleep(std::time::Duration::from_millis(17));
                server.step();
            }
        },
        Options::MultiplayerClient { address } => {
            let event_loop = EventLoop::new();
            let (mut renderer, mut buffer_renderer) = renderer::Renderer::new(&event_loop).await;

            let db = setup_rendering_systems(db, &[]);

            println!("{:?}", db);

            let mut dispatcher = db.build();

            let client = std::sync::Arc::new(networking::Client::new(address));

            let update_client = client.clone();
            let handle = std::thread::spawn(move || update_client.update_loop());

            event_loop.run(move |event, _, control_flow| match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(size) => {
                        renderer.resize(size.width as u32, size.height as u32);
                        buffer_renderer.set_window_size(size.width, size.height);
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
        
                        use networking::Key;
                        match code {
                            VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                            VirtualKeyCode::Left => client.send(Key::Left, pressed),
                            VirtualKeyCode::Right => client.send(Key::Right, pressed),
                            VirtualKeyCode::Up => client.send(Key::Up, pressed),
                            VirtualKeyCode::Down => client.send(Key::Down, pressed),
                            VirtualKeyCode::Z => client.send(Key::Fire, pressed),
                            _ => {},
                        }
                    }
                    _ => {}
                },
                Event::MainEventsCleared => {
                    for (image, pos) in &*client.state() {
                        buffer_renderer.render_sprite(*image, *pos, [0.0; 4]);
                    }
                    renderer.request_redraw();
                },
                Event::RedrawRequested(_) => renderer.render(&mut buffer_renderer),
                _ => {}
            });
        }
    }
}

fn run_event_loop(
    event_loop: EventLoop<()>,
    mut renderer: renderer::Renderer,
    mut world: World,
    mut dispatcher: Dispatcher<'static, 'static>
) {
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

fn setup_world() -> specs::World {
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

    world.insert(resources::KeyPresses(vec![]));
    world.insert(resources::KeyboardState::default());
    world.insert(resources::GameTime(0.0));
    world.insert(resources::BulletSpawner::default());
    world.insert(resources::DamageTracker::default());

    world
}

fn setup_game_systems<'a, 'b>(dispatch_builder: DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b> {
    dispatch_builder
        .with(systems::KillOffscreen, "KillOffscreen", &[])
        .with(systems::MoveEntities, "MoveEntities", &[])
        .with(systems::HandleKeypresses, "HandleKeypresses", &[])
        .with(systems::Control, "Control", &[])
        .with(systems::FireBullets, "FireBullets", &[])
        .with(systems::SpawnBullets, "SpawnBullets", &[])
        .with(systems::RepeatBackgroundLayers, "RepeatBackgroundLayers", &[])
        .with(systems::TickTime, "TickTime", &[])
        .with(systems::AddOnscreen, "AddOnscreen", &[])
        .with(systems::Collisions, "Collisions", &[])
        .with(systems::ApplyCollisions, "ApplyCollisions", &["Collisions"])
        .with(systems::ExplosionImages, "ExplosionImages", &["ApplyCollisions"])
}

fn setup_rendering_systems<'a, 'b>(dispatch_builder: DispatcherBuilder<'a, 'b>, dependencies: &[&str]) -> DispatcherBuilder<'a, 'b> {
    dispatch_builder
        .with(systems::RenderSprite, "RenderSprite", dependencies)
        .with(systems::RenderText, "RenderText", &["RenderSprite"])
        //.with(systems::RenderHitboxes, "RenderHitboxes", &["RenderSprite"])
}
