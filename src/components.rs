use cgmath::Vector2;
use specs::*;

use crate::graphics::Image as GraphicsImage;

#[derive(Component)]
pub struct Image(u16);

impl Image {
    pub fn coordinates(&self) -> (f32, f32, f32, f32) {
        let (x, y, w, h) = GraphicsImage::from_u16(self.0).coordinates();
        let size = GraphicsImage::from_u16(self.0).image_size() as f32;
        (x as f32 / size, y as f32 / size, w as f32 / size, h as f32 / size)
    }

    pub fn size(&self) -> (f32, f32) {
        let (_, _, w, h) = GraphicsImage::from_u16(self.0).coordinates();
        (w as f32, h as f32)
    }

    pub fn from(image: GraphicsImage) -> Self {
        Self(image.to_u16())
    }
}

#[derive(Component)]
pub struct BackgroundLayer;

#[derive(Component)]
pub struct Position(pub Vector2<f32>);

#[derive(Component)]
pub enum Movement {
    Linear(Vector2<f32>),
    Falling(f32)
}

#[derive(Component)]
pub struct DieOffscreen;
