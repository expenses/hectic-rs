use specs::prelude::*;
use cgmath::Vector2;
use crate::{WIDTH, HEIGHT, resources::*, components::*, renderer::BufferRenderer as Renderer, graphics::Image as GraphicsImage};

pub struct RepeatBackgroundLayers;

impl<'a> System<'a> for RepeatBackgroundLayers {
    type SystemData = (ReadStorage<'a, BackgroundLayer>, ReadStorage<'a, Image>, WriteStorage<'a, Position>);

    fn run(&mut self, (layer, image, mut pos): Self::SystemData) {
        for (_layer, image, pos) in (&layer, &image, &mut pos).join() {
            let size = image.size();
            if pos.0.y > size.y * 2.0 {
                pos.0.y -= size.y * 4.0;
            }
        }
    }
}

pub struct RenderSprite;

impl<'a> System<'a> for RenderSprite {
    type SystemData = (Entities<'a>, ReadStorage<'a, Position>, ReadStorage<'a, Image>, ReadStorage<'a, Invulnerability>, ReadStorage<'a, FrozenUntil>, Read<'a, GameTime>, Write<'a, Renderer>);

    fn run(&mut self, (entities, pos, image, invul, frozen, time, mut renderer): Self::SystemData) {
        for (entity, pos, image, _) in (&entities, &pos, &image, !&frozen).join() {
            let overlay = invul.get(entity)
                .filter(|invul| invul.is_invul(time.total_time))
                .map(|_| [1.0, 1.0, 1.0, 0.2])
                .unwrap_or([0.0; 4]);

            renderer.render_sprite(*image, pos.0, overlay);
        }
    }
}

pub struct RenderHitboxes;

impl<'a> System<'a> for RenderHitboxes {
    type SystemData = (ReadStorage<'a, Position>, ReadStorage<'a, Hitbox>, Write<'a, Renderer>, Read<'a, ControlsState>);

    fn run(&mut self, (pos, hit, mut renderer, ctrl_state): Self::SystemData) {
        if !ctrl_state.debug.pressed {
            return;
        }

        for (pos, hit) in (&pos, &hit).join() {
            renderer.render_box(pos.0, hit.0, [1.0, 0.0, 0.0, 0.5]);
        }
    }
}

pub struct RenderPauseBackground;

impl<'a> System<'a> for RenderPauseBackground {
    type SystemData = Write<'a, Renderer>;

    fn run(&mut self, mut renderer: Self::SystemData) {
        renderer.render_box(Vector2::new(WIDTH / 2.0, HEIGHT / 2.0), Vector2::new(WIDTH, HEIGHT), [0.0, 0.0, 0.0, 0.5]);
    }
}

pub struct RenderText;

impl<'a> System<'a> for RenderText {
    type SystemData = (ReadStorage<'a, Position>, ReadStorage<'a, Text>, Write<'a, Renderer>);

    fn run(&mut self, (pos, text, mut renderer): Self::SystemData) {
        for (pos, text) in (&pos, &text).join() {
            renderer.render_text(text, pos.0, [1.0; 4]);
        }
    }
}

pub struct RenderUI;

impl<'a> System<'a> for RenderUI {
    type SystemData = (ReadStorage<'a, Player>, ReadStorage<'a, Health>, Write<'a, Renderer>);

    fn run(&mut self, (player, health, mut renderer): Self::SystemData) {
        let mut join = (&player, &health).join().map(|(_, health)| health.0);

        if let Some(health) = join.next() {
            renderer.render_text(&Text {
                text: format!("Health: {}", health),
                font: 1,
                layout: wgpu_glyph::Layout::default()
            }, Vector2::new(0.0, 0.0), [1.0; 4]);
        }

        if let Some(health) = join.next() {
            renderer.render_text(&Text {
                text: format!("Health: {}", health),
                font: 1,
                layout: wgpu_glyph::Layout::default()
            }, Vector2::new(0.0, 20.0), [1.0; 4]);
        }
    }
}

pub struct RenderMenu;

impl<'a> System<'a> for RenderMenu {
    type SystemData = (Write<'a, Renderer>, Write<'a, Mode>, Read<'a, ControlsState>);

    fn run(&mut self, (mut renderer, mut mode, ctrl_state): Self::SystemData) {
        if let Some(menu) = mode.as_menu(&ctrl_state) {
            renderer.render_text(&Text::title(&menu.title), Vector2::new(WIDTH / 2.0, 40.0), [1.0; 4]);

            let mut x = 190.0;

            for (i, item) in menu.items.iter().enumerate() {
                renderer.render_text(&Text {
                    text: if i == *menu.selected { format!("> {}", item.text) } else { item.text.to_string() },
                    font: 1,
                    layout: wgpu_glyph::Layout::default().h_align(wgpu_glyph::HorizontalAlign::Center)
                }, Vector2::new(WIDTH / 2.0, x), if item.active { [1.0; 4] } else { [0.5, 0.5, 0.5, 1.0] });
                x += 20.0;
            }
        }
    }
}

pub struct ExplosionImages;

impl<'a> System<'a> for ExplosionImages {
    type SystemData = (Entities<'a>, ReadStorage<'a, Explosion>, WriteStorage<'a, Image>, Read<'a, GameTime>);

    fn run(&mut self, (entities, explosion, mut image, time): Self::SystemData) {
        for (entity, explosion) in (&entities, &explosion).join() {
            let images = [
                Image::from(GraphicsImage::Explosion1),
                Image::from(GraphicsImage::Explosion2),
                Image::from(GraphicsImage::Explosion3),
                Image::from(GraphicsImage::Explosion4),
                Image::from(GraphicsImage::Explosion5),
                Image::from(GraphicsImage::Explosion6),
            ];

            let index = ((time.total_time - explosion.0) / 0.5 * images.len() as f32) as usize;

            if index < images.len() {
                image.insert(entity, images[index]).unwrap();
            } else {
                entities.delete(entity).unwrap();
            }
        }
    }
}
