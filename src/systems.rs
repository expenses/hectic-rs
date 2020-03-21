use specs::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::Vertex;
use winit::{
    event::VirtualKeyCode,
};
use cgmath::Vector2;

use crate::graphics::Image as GraphicsImage;

const WIDTH: f32 = 480.0;
const HEIGHT: f32 = 640.0;
const PLAYER_SPEED: f32 = 250.0 / 60.0;
const PLAYER_BULLET_SPEED: f32 = 500.0 / 60.0;

pub fn render_sprite(sprite: &Image, mut x: f32, mut y: f32, renderer: &mut Renderer) {
    let (pos_x, pos_y, width, height) = sprite.coordinates();

    let (s_w, s_h) = renderer.window_size;
    x *= 2.0;
    y *= 2.0;
    x -= s_w;
    y -= s_h;
    x /= s_w;// / renderer.dpi_factor;
    y /= s_h;// / renderer.dpi_factor;
    
    let (mut sp_w, mut sp_h) = sprite.size();
    sp_w *= 2.0;// * renderer.dpi_factor;
    sp_h *= 2.0;// * renderer.dpi_factor;
    sp_w /= s_w;
    sp_h /= s_h;

    let len = renderer.vertices.len() as i16;

    renderer.vertices.extend_from_slice(&[
        Vertex{pos: [x + sp_w, y - sp_h], uv: [pos_x + width, pos_y]},
        Vertex{pos:[x - sp_w, y - sp_h], uv: [pos_x, pos_y]},
        Vertex{pos: [x - sp_w, y + sp_h], uv: [pos_x, pos_y + height]},
        Vertex{pos: [x + sp_w, y + sp_h], uv: [pos_x + width, pos_y + height]},
    ]);

    renderer.indices.extend_from_slice(&[len, len + 1, len + 2, len + 2, len + 3, len]);
}

pub struct RepeatBackgroundLayers;

impl<'a> System<'a> for RepeatBackgroundLayers {
    type SystemData = (ReadStorage<'a, BackgroundLayer>, ReadStorage<'a, Image>, WriteStorage<'a, Position>);

    fn run(&mut self, (layer, image, mut pos): Self::SystemData) {
        for (_layer, image, pos) in (&layer, &image, &mut pos).join() {
            let (_, height) = image.size();
            if pos.0.y > height * 2.0 {
                pos.0.y -= height * 4.0;
            }
        }
    }
}

pub struct RenderSprite;

impl<'a> System<'a> for RenderSprite {
    type SystemData = (ReadStorage<'a, Position>, ReadStorage<'a, Image>, Write<'a, Renderer>);

    fn run(&mut self, (pos, image, mut renderer): Self::SystemData) {
        for (pos, image) in (&pos, &image).join() {
            render_sprite(image, pos.0.x, pos.0.y, &mut renderer)
        }
    }
}

pub struct MoveEntities;

impl<'a> System<'a> for MoveEntities {
    type SystemData = (WriteStorage<'a, Position>, WriteStorage<'a, Movement>, ReadStorage<'a, FrozenUntil>);

    fn run(&mut self, (mut pos, mut mov, frozen): Self::SystemData) {
        for (mut pos, mov, frozen) in (&mut pos, &mut mov, !&frozen).join() {
            match mov {
                Movement::Linear(vector) => pos.0 = pos.0 + *vector,
                Movement::Falling(speed) => {
                    pos.0.y -= *speed;
                    *speed += 0.15;
                },
                Movement::FollowCurve(curve) => {
                    pos.0 = curve.step(pos.0);
                }
            }
        }
    }
}

pub struct HandleKeypresses;

impl<'a> System<'a> for HandleKeypresses {
    type SystemData = (Write<'a, KeyPresses>, Write<'a, Controls>);

    fn run(&mut self, (mut presses, mut controls): Self::SystemData) {
        for (key, pressed) in presses.0.drain(..) {
            match key {
                VirtualKeyCode::Left => controls.left = pressed,
                VirtualKeyCode::Right => controls.right = pressed,
                VirtualKeyCode::Down => controls.down = pressed,
                VirtualKeyCode::Up => controls.up = pressed,
                VirtualKeyCode::Z => controls.fire = pressed,
                _ => {}
            }
        }
    }
}

pub struct Control;

fn min(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

fn max(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

impl<'a> System<'a> for Control {
    type SystemData = (
        Entities<'a>, Read<'a, Controls>, ReadStorage<'a, Controllable>,
        WriteStorage<'a, Position>, WriteStorage<'a, Image>, WriteStorage<'a, Movement>, WriteStorage<'a, DieOffscreen>,
    );

    fn run(&mut self, (entity, controls, controllable, mut position, mut image, mut movement, mut dieoffscreen): Self::SystemData) {
        let bullet_positions: Vec<_> = (&controllable, &mut position).join()
            .filter_map(|(_, mut pos)| {
                if controls.left {
                    pos.0.x = max(pos.0.x - PLAYER_SPEED, 0.0);
                }
    
                if controls.right {
                    pos.0.x = min(pos.0.x + PLAYER_SPEED, WIDTH);
                }
    
                if controls.up {
                    pos.0.y = max(pos.0.y - PLAYER_SPEED, 0.0);
                }
    
                if controls.down {
                    pos.0.y = min(pos.0.y + PLAYER_SPEED, HEIGHT);
                }

                if controls.fire {
                    Some(pos.0)
                } else {
                    None
                }
            })
            .collect();

        bullet_positions.into_iter().for_each(|pos| {
            entity.build_entity()
                .with(Position(pos), &mut position)
                .with(Image::from(GraphicsImage::PlayerBullet), &mut image)
                .with(Movement::Linear(Vector2::new(0.0, -PLAYER_BULLET_SPEED)), &mut movement)
                .with(DieOffscreen, &mut dieoffscreen)
                .build();
        });
    }
}

pub struct TickTime;

impl<'a> System<'a> for TickTime {
    type SystemData = (Entities<'a>, Write<'a, GameTime>, WriteStorage<'a, FrozenUntil>);

    fn run(&mut self, (entities, mut game_time, mut frozen): Self::SystemData) {
        game_time.0 += 1.0 / 60.0;
        
        for (_, entry) in (&entities, frozen.entries()).join() {
            if let specs::storage::StorageEntry::Occupied(entry) = entry {
                if entry.get().0 <= game_time.0 {
                    entry.remove();
                }
            }
        }
    } 
}

pub struct KillOffscreen;

impl<'a> System<'a> for KillOffscreen {
    type SystemData = (Entities<'a>, ReadStorage<'a, Position>, ReadStorage<'a, BeenOnscreen>, ReadStorage<'a, Image>);

    fn run(&mut self, (entities, pos, been_onscreen, image): Self::SystemData) {
        for (entity, pos, _, image) in (&entities, &pos, &been_onscreen, &image).join() {
            if !(is_onscreen(pos, image)) {
                entities.delete(entity);
            }
        }
    }
}

pub struct AddOnscreen;

impl<'a> System<'a> for AddOnscreen {
    type SystemData = (Entities<'a>, ReadStorage<'a, Position>, ReadStorage<'a, Image>, ReadStorage<'a, DieOffscreen>, WriteStorage<'a, BeenOnscreen>);

    fn run(&mut self, (entities, pos, image, die_offscreen, mut been_onscreen): Self::SystemData) {
        for (entity, pos, image, _) in (&entities, &pos, &image, &die_offscreen).join() {
            if is_onscreen(pos, image) {
                been_onscreen.insert(entity, BeenOnscreen).unwrap();
            }
        }
    }
}

fn is_onscreen(pos: &Position, image: &Image) -> bool {
    let (w, h) = image.size();
    let (w, h) = (w / 2.0, h / 2.0);
    !(pos.0.y + h <= 0.0 || pos.0.y - h >= HEIGHT || pos.0.x + w <= 0.0 || pos.0.x - w >= WIDTH)
}
