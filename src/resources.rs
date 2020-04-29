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
pub struct DamageTracker(pub Vec<Damage>);

pub struct Damage {
    pub friendly: specs::Entity,
    pub enemy: specs::Entity,
    pub position: Vector2<f32>,
}

#[derive(Default)]
pub struct PlayerPositions(pub Vec<Vector2<f32>>);

impl PlayerPositions {
    pub fn random(&self, rng: &mut rand::rngs::ThreadRng) -> Vector2<f32> {
        let index = rng.gen_range(0, self.0.len());
        self.0[index]
    }
}
