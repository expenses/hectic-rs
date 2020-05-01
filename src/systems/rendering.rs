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
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>, ReadStorage<'a, Image>, ReadStorage<'a, Invulnerability>, ReadStorage<'a, FrozenUntil>, ReadStorage<'a, BackgroundLayer>,
        Read<'a, GameTime>, Write<'a, Renderer>
    );

    fn run(&mut self, (entities, pos, image, invul, frozen, bg, time, mut renderer): Self::SystemData) {
        let mut background_positions: Vec<_> = (&pos, &image, &bg).join().collect();
        background_positions.sort_unstable_by_key(|(_, _, bg)| bg.depth);

        for (pos, image, _) in background_positions.drain(..) {
            renderer.render_sprite(*image, pos.0, [0.0; 4]);
        }

        for (entity, pos, image, _, _) in (&entities, &pos, &image, !&frozen, !&bg).join() {
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
            let mut hitbox = hit.0;
            hitbox.x = hitbox.x.max(2.0);
            hitbox.y = hitbox.y.max(2.0);
            renderer.render_box(pos.0, hitbox, [1.0, 0.0, 0.0, 0.5]);
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
    type SystemData = (ReadStorage<'a, Player>, ReadStorage<'a, Health>, ReadStorage<'a, PowerBar>, Write<'a, Renderer>);

    fn run(&mut self, (player, health, bar, mut renderer): Self::SystemData) {
        let mut join = (&player, &health, &bar).join().map(|(_, health, bar)| (health.0, bar));

        const MAX_BAR_HEIGHT: f32 = 32.0;
        const BAR_WIDTH: f32 = 16.0;
        const PADDING: f32 = 4.0;
        const PADDED_MAX_BAR_HEIGHT: f32 = MAX_BAR_HEIGHT - PADDING;
        const BAR_DIMENSIONS: Vector2<f32> = Vector2::new(BAR_WIDTH, MAX_BAR_HEIGHT);

        if let Some((health, bar)) = join.next() {
            renderer.render_text(&Text {
                text: health.to_string(),
                font: 1,
                layout: wgpu_glyph::Layout::default().v_align(wgpu_glyph::VerticalAlign::Center)
            }, Vector2::new(60.0, HEIGHT - 30.0), [1.0; 4]);

            renderer.render_sprite(Image::from(GraphicsImage::Portrait), Vector2::new(30.0, HEIGHT - 30.0), [0.0; 4]);

            let perc = bar.perc();
            let missing = (PADDED_MAX_BAR_HEIGHT - (perc * PADDED_MAX_BAR_HEIGHT)) / 2.0;

            renderer.render_box(Vector2::new(80.0, HEIGHT - 30.0), BAR_DIMENSIONS, [0.0, 0.0, 0.0, 1.0]);
            renderer.render_box(Vector2::new(80.0, HEIGHT - 30.0 + missing), Vector2::new(BAR_WIDTH - PADDING, perc * PADDED_MAX_BAR_HEIGHT), [0.5, 0.125, 0.125, 1.0]);
        }

        if let Some((health, bar)) = join.next() {
            renderer.render_text(&Text {
                text: health.to_string(),
                font: 1,
                layout: wgpu_glyph::Layout::default().v_align(wgpu_glyph::VerticalAlign::Center).h_align(wgpu_glyph::HorizontalAlign::Right)
            }, Vector2::new(WIDTH - 60.0, HEIGHT - 30.0), [1.0; 4]);

            renderer.render_sprite(Image::from(GraphicsImage::Portrait), Vector2::new(WIDTH - 30.0, HEIGHT - 30.0), [0.0; 4]);

            let perc = bar.perc();
            let missing = (PADDED_MAX_BAR_HEIGHT - (perc * PADDED_MAX_BAR_HEIGHT)) / 2.0;

            renderer.render_box(Vector2::new(WIDTH - 80.0, HEIGHT - 30.0), BAR_DIMENSIONS, [0.0, 0.0, 0.0, 1.0]);
            renderer.render_box(Vector2::new(WIDTH - 80.0, HEIGHT - 30.0 + missing), Vector2::new(BAR_WIDTH - PADDING, perc * PADDED_MAX_BAR_HEIGHT), [0.5, 0.125, 0.125, 1.0]);
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

pub struct RenderBombs;

impl<'a> System<'a> for RenderBombs {
    type SystemData = (Write<'a, Renderer>, ReadStorage<'a, Position>, ReadStorage<'a, Circle>);

    fn run(&mut self, (mut renderer, pos, circle): Self::SystemData) {
        for (pos, circle) in (&pos, &circle).join() {
            renderer.render_circle(pos.0, circle.radius);
        }
    }
}