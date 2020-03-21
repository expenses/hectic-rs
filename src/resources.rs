use cgmath::Vector2;
use winit::event::VirtualKeyCode;

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

#[derive(Default)]
pub struct GameTime(pub f32);
