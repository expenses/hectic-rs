use specs::prelude::*;

use crate::components::*;
use crate::resources::*;

use cgmath::Vector2;
use rand::Rng;

use crate::renderer::BufferRenderer as Renderer;

use crate::graphics::Image as GraphicsImage;

const WIDTH: f32 = 480.0;
const HEIGHT: f32 = 640.0;
const PLAYER_SPEED: f32 = 250.0 / 60.0;
const PLAYER_BULLET_SPEED: f32 = 500.0 / 60.0;

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
            renderer.render_box(pos.0, hit.0);
        }
    }
}

pub struct RenderPauseScreen;

impl<'a> System<'a> for RenderPauseScreen {
    type SystemData = (Write<'a, Renderer>);

    fn run(&mut self, (mut renderer): Self::SystemData) {
        renderer.render_text(&Text::title("Paused"), Vector2::new(WIDTH / 2.0, 40.0));
    }
}

pub struct RenderText;

impl<'a> System<'a> for RenderText {
    type SystemData = (ReadStorage<'a, Position>, ReadStorage<'a, Text>, Write<'a, Renderer>);

    fn run(&mut self, (pos, text, mut renderer): Self::SystemData) {
        for (pos, text) in (&pos, &text).join() {
            renderer.render_text(text, pos.0);
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
            }, Vector2::new(0.0, 0.0));
        }

        if let Some(health) = join.next() {
            renderer.render_text(&Text {
                text: format!("Health: {}", health),
                font: 1,
                layout: wgpu_glyph::Layout::default()
            }, Vector2::new(0.0, 20.0));
        }
    }
}


pub struct MoveEntities;

impl<'a> System<'a> for MoveEntities {
    type SystemData = (WriteStorage<'a, Position>, WriteStorage<'a, Movement>, ReadStorage<'a, FrozenUntil>, Read<'a, GameTime>);

    fn run(&mut self, (mut pos, mut mov, frozen, game_time): Self::SystemData) {
        for (mut pos, mov, _) in (&mut pos, &mut mov, !&frozen).join() {
            match mov {
                Movement::Linear(vector) => pos.0 += *vector,
                Movement::Falling(speed) => {
                    pos.0.y -= *speed;
                    *speed += 0.15;
                },
                Movement::FollowCurve(curve) => {
                    pos.0 = curve.step(pos.0);
                },
                Movement::FiringMove(speed, return_time, stop_y) => {
                    if *return_time <= game_time.total_time {
                        pos.0.y -= *speed;
                    } else {
                        pos.0.y = min(pos.0.y + *speed, *stop_y);
                    }
                }
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
        Read<'a, ControlsState>, Read<'a, GameTime>, Write<'a, BulletSpawner>,
        ReadStorage<'a, Player>, WriteStorage<'a, Position>, WriteStorage<'a, Cooldown>
    );

    fn run(&mut self, (ctrl_state, time, mut spawner, player, mut position, mut cooldown): Self::SystemData) {
        for (player, mut pos, cooldown) in (&player, &mut position, &mut cooldown).join() {
            let player_ctrl_state = ctrl_state.get(*player);

            if player_ctrl_state.left.pressed {
                pos.0.x = max(pos.0.x - PLAYER_SPEED, 0.0);
            }

            if player_ctrl_state.right.pressed {
                pos.0.x = min(pos.0.x + PLAYER_SPEED, WIDTH);
            }

            if player_ctrl_state.up.pressed {
                pos.0.y = max(pos.0.y - PLAYER_SPEED, 0.0);
            }

            if player_ctrl_state.down.pressed {
                pos.0.y = min(pos.0.y + PLAYER_SPEED, HEIGHT);
            }

            if player_ctrl_state.fire.pressed && cooldown.is_ready(time.total_time) {
                for direction in &[-0.2_f32, -0.1, 0.0, 0.1, 0.2] {
                    spawner.0.push(BulletToBeSpawned {
                        pos: pos.0,
                        image: Image::from(GraphicsImage::PlayerBullet),
                        velocity: Vector2::new(direction.sin(), -direction.cos()) * PLAYER_BULLET_SPEED,
                        enemy: false,
                    });
                }
            }
        }
    }
}

pub struct TickTime;

impl<'a> System<'a> for TickTime {
    type SystemData = (Entities<'a>, Write<'a, GameTime>, WriteStorage<'a, FrozenUntil>);

    fn run(&mut self, (entities, mut game_time, mut frozen): Self::SystemData) {
        game_time.total_time += 1.0 / 60.0;

        for (_, entry) in (&entities, frozen.entries()).join() {
            if let specs::storage::StorageEntry::Occupied(entry) = entry {
                if entry.get().0 <= game_time.total_time {
                    entry.remove();
                }
            }
        }
    } 
}

pub struct StartTowardsPlayer;

impl<'a> System<'a> for StartTowardsPlayer {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, FrozenUntil>, ReadStorage<'a, Position>,
        WriteStorage<'a, TargetPlayer>, WriteStorage<'a, Movement>,
        Read<'a, PlayerPositions>,
    );

    fn run(&mut self, (entities, frozen, pos, mut target, mut movement, player_positions): Self::SystemData) {
        let mut rng = rand::thread_rng();
        
        for (entity, target, pos, _) in (&entities, target.entries(), &pos, !&frozen).join() {
            if let specs::storage::StorageEntry::Occupied(target) = target {
                let speed = target.get().0;
                target.remove();

                let player = player_positions.random(&mut rng);
                let rotation = (player.y - pos.0.y).atan2(player.x - pos.0.x);

                movement.insert(entity, Movement::Linear(Vector2::new(rotation.cos() * speed, rotation.sin() * speed)))
                    .unwrap();
            }
        }
    }
}

pub struct SetPlayerPositions;

impl<'a> System<'a> for SetPlayerPositions {
    type SystemData = (ReadStorage<'a, Position>, ReadStorage<'a, Player>, Write<'a, PlayerPositions>);

    fn run(&mut self, (pos, player, mut positions): Self::SystemData) {
        positions.0.clear();

        for (pos, _) in (&pos, &player).join() {
            positions.0.push(pos.0);
        }
    }
}

pub struct KillOffscreen;

impl<'a> System<'a> for KillOffscreen {
    type SystemData = (Entities<'a>, ReadStorage<'a, Position>, ReadStorage<'a, BeenOnscreen>, ReadStorage<'a, Image>);

    fn run(&mut self, (entities, pos, been_onscreen, image): Self::SystemData) {
        for (entity, pos, _, image) in (&entities, &pos, &been_onscreen, &image).join() {
            if !(is_onscreen(pos, *image)) {
                entities.delete(entity).unwrap();
            }
        }
    }
}

pub struct AddOnscreen;

impl<'a> System<'a> for AddOnscreen {
    type SystemData = (Entities<'a>, ReadStorage<'a, Position>, ReadStorage<'a, Image>, ReadStorage<'a, DieOffscreen>, WriteStorage<'a, BeenOnscreen>);

    fn run(&mut self, (entities, pos, image, die_offscreen, mut been_onscreen): Self::SystemData) {
        for (entity, pos, image, _) in (&entities, &pos, &image, &die_offscreen).join() {
            if is_onscreen(pos, *image) {
                been_onscreen.insert(entity, BeenOnscreen).unwrap();
            }
        }
    }
}

fn is_onscreen(pos: &Position, image: Image) -> bool {
    let size = image.size() / 2.0;
    !(pos.0.y + size.y <= 0.0 || pos.0.y - size.y >= HEIGHT || pos.0.x + size.x <= 0.0 || pos.0.x - size.x >= WIDTH)
}

pub struct FireBullets;

impl<'a> System<'a> for FireBullets {
    type SystemData = (
        ReadStorage<'a, Position>, ReadStorage<'a, FiresBullets>, WriteStorage<'a, Cooldown>, ReadStorage<'a, FrozenUntil>,
        Write<'a, BulletSpawner>, Read<'a, GameTime>, Read<'a, PlayerPositions>,
    );

    fn run(&mut self, (pos, fires, mut cooldown, frozen, mut spawner, time, player_positions): Self::SystemData) {
        let mut rng = rand::thread_rng();

        for (pos, fires, cooldown, _) in (&pos, &fires, &mut cooldown, !&frozen).join() {
            if cooldown.is_ready(time.total_time) {
                match fires.method {
                    FiringMethod::AtPlayer(total, spread) => {
                        let player = player_positions.random(&mut rng);

                        // Get the rotation to the player
                        let rotation = (player.y - pos.0.y).atan2(player.x - pos.0.x);

                        for i in 0 .. total {
                            let mid_point = (total - 1) as f32 / 2.0;
                            let rotation_difference = spread * (mid_point - i as f32) / total as f32;

                            spawner.0.push(fires.bullet_to_be_spawned(pos.0, rotation + rotation_difference));
                        }
                    }
                }
            }
        }
    }
}

pub struct SpawnBullets;

impl<'a> System<'a> for SpawnBullets {
    type SystemData = (
        Entities<'a>, Write<'a, BulletSpawner>,
        WriteStorage<'a, Position>, WriteStorage<'a, Image>, WriteStorage<'a, Movement>,
        WriteStorage<'a, DieOffscreen>, WriteStorage<'a, Friendly>, WriteStorage<'a, Enemy>,
        WriteStorage<'a, Hitbox>,
        WriteStorage<'a, Health>,
    );

    fn run(&mut self, (entities, mut spawner, mut pos, mut image, mut mov, mut dieoffscreen, mut friendly, mut enemy, mut hitbox, mut health): Self::SystemData) {
        for bullet in spawner.0.drain(..) {
            if bullet.enemy {
                entities.build_entity()
                    .with(Enemy, &mut enemy)
            } else {
                entities.build_entity()
                    .with(Friendly, &mut friendly)
            }
                .with(Position(bullet.pos), &mut pos)
                .with(bullet.image, &mut image)
                .with(Movement::Linear(bullet.velocity), &mut mov)
                .with(DieOffscreen, &mut dieoffscreen)
                .with(Hitbox(Vector2::new(0.0, 0.0)), &mut hitbox)
                .with(Health(1), &mut health)
                .build();
        }
    }
}

pub struct Collisions;

impl<'a> System<'a> for Collisions {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>, ReadStorage<'a, Friendly>, ReadStorage<'a, Enemy>, ReadStorage<'a, Hitbox>,
        ReadStorage<'a, FrozenUntil>,
        Write<'a, DamageTracker>,
    );

    fn run(&mut self, (entities, pos, friendly, enemy, hitbox, frozen, mut damage_tracker): Self::SystemData) {
        (&entities, &pos, &hitbox, &friendly).join()
            .flat_map(|friendly| {
                (&entities, &pos, &hitbox, !&frozen, &enemy).join()
                    .map(move |enemy| (friendly, enemy))
            })
            .for_each(|((f_entity, f_pos, f_hitbox, _), (e_entity, e_pos, e_hitbox, _, _))| {
                if let Some(hit_pos) = is_touching(f_pos.0, f_hitbox.0, e_pos.0, e_hitbox.0) {
                    damage_tracker.0.push(Damage {
                        friendly: f_entity,
                        enemy: e_entity,
                        position: hit_pos,
                    });
                }
            });
    }
}

pub struct ApplyCollisions;

impl<'a> System<'a> for ApplyCollisions {
    type SystemData = (
        Entities<'a>, Write<'a, DamageTracker>, Read<'a, GameTime>,
        WriteStorage<'a, Health>, WriteStorage<'a, Position>, WriteStorage<'a, Explosion>, WriteStorage<'a, Invulnerability>,
    );

    fn run(&mut self, (entities, mut damage_tracker, time, mut health, mut pos, mut explosion, mut invul): Self::SystemData) {
        let mut rng = rand::thread_rng();

        for mut damage in damage_tracker.0.drain(..) {
            let player_triggered_invul = damage_entity(damage.friendly, &entities, &mut health, &mut invul, time.total_time);
            if player_triggered_invul {
                damage_entity(damage.enemy, &entities, &mut health, &mut invul, time.total_time);

                damage.position.x += rng.gen_range(-5.0, 5.0);
                damage.position.y += rng.gen_range(-5.0, 5.0);
    
                entities.build_entity()
                    .with(Position(damage.position), &mut pos)
                    .with(Explosion(time.total_time), &mut explosion)
                    .build();
            }
        }
    }
}

fn damage_entity(entity: Entity, entities: &Entities, health: &mut WriteStorage<Health>, invul: &mut WriteStorage<Invulnerability>, time: f32) -> bool {
    if let Some(health) = health.get_mut(entity) {
        let invul = invul.get_mut(entity).map(|invul| invul.can_damage(time)).unwrap_or(true);

        if invul {
            health.0 = health.0.saturating_sub(1);

            if health.0 == 0 {
                entities.delete(entity).unwrap();
            }
        }

        invul
    } else {
        false
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

fn is_touching(pos_a: Vector2<f32>, hit_a: Vector2<f32>, pos_b: Vector2<f32>, hit_b: Vector2<f32>) -> Option<Vector2<f32>> {
    if hit_a == Vector2::new(0.0, 0.0) && hit_b == Vector2::new(0.0, 0.0) {
        return None;
    }

    let a_t_l = pos_a - hit_a / 2.0;
    let a_b_r = pos_a + hit_a / 2.0;
    
    let b_t_l = pos_b - hit_b / 2.0;
    let b_b_r = pos_b + hit_b / 2.0;
    
    let is_touching = !(
        a_t_l.x > b_b_r.x  || a_b_r.x  < b_t_l.x ||
        a_t_l.y  > b_b_r.y || a_b_r.y < b_t_l.y
    );

    if is_touching {
        Some(if hit_a.x * hit_a.y > hit_b.x * hit_b.y {
            pos_b
        } else {
            pos_a
        })
    } else {
        None
    }
}
