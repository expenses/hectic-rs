use winit::event::VirtualKeyCode;
use cgmath::Vector2;

#[derive(Default)]
pub struct KeyPresses(pub Vec<(VirtualKeyCode, bool)>);

#[derive(Default)]
pub struct KeyboardState(pub std::collections::HashMap<VirtualKeyCode, bool>);

impl KeyboardState {
    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.0.get(&key).cloned().unwrap_or(false)
    }
}

#[derive(Default)]
pub struct GameTime(pub f32);

#[derive(Default)]
pub struct BulletSpawner(pub Vec<BulletToBeSpawned>);

pub struct BulletToBeSpawned {
    pub pos: Vector2<f32>,
    pub image: crate::components::Image,
    pub velocity: Vector2<f32>,
    pub enemy: bool,
}

#[derive(Default)]
pub struct DamageTracker(pub Vec<(specs::Entity, specs::Entity, Vector2<f32>)>);
