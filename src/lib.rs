
mod graphics;
mod components;
pub mod resources;
pub mod systems;
pub mod stages;
mod renderer;

use cgmath::Vector2;
use specs::prelude::*;

const WIDTH: f32 = 480.0;
const HEIGHT: f32 = 640.0;
const DIMENSIONS: Vector2<f32> = Vector2::new(WIDTH, HEIGHT);
const ZERO: Vector2<f32> = Vector2::new(0.0, 0.0);
const MIDDLE: Vector2<f32> = Vector2::new(WIDTH / 2.0, HEIGHT / 2.0);

pub fn register_components(world: &mut World) {
    world.register::<components::Position>();
    world.register::<components::Image>();
    world.register::<components::Velocity>();
    world.register::<components::Falling>();
    world.register::<components::FollowCurve>();
    world.register::<components::FiringMove>();
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
    world.register::<components::MoveTowards>();
    world.register::<components::Boss>();
    world.register::<components::ColourOverlay>();
}
