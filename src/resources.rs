use cgmath::Vector2;
use winit::event::VirtualKeyCode;

pub struct PlayerPosition(pub Vector2<f32>);

impl Default for PlayerPosition {
    fn default() -> Self {
        Self(Vector2::new(0.0, 0.0))
    }
}

pub struct PlayerHealth(pub u8);

#[derive(Default)]
pub struct KeyPresses(pub Vec<(VirtualKeyCode, bool)>);

#[derive(Default)]
pub struct Controls {
    pub left: bool,
    pub up: bool,
    pub down: bool,
    pub right: bool,
    pub fire: bool,
}

#[derive(Default)]
pub struct Renderer {
    pub vertices: Vec<crate::Vertex>,
    pub indices: Vec<i16>,
    pub dpi_factor: f32,
    pub window_size: (f32, f32)
}
