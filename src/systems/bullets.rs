use specs::prelude::*;
use cgmath::{Vector2, MetricSpace, InnerSpace};
use rand::Rng;
use crate::{WIDTH, HEIGHT, resources::*, components::*, graphics::Image as GraphicsImage};
use super::{is_touching, build_bullet};

pub struct FireBullets;

impl<'a> System<'a> for FireBullets {
    type SystemData = (
        Entities<'a>, ReadStorage<'a, Position>, ReadStorage<'a, FiresBullets>, WriteStorage<'a, Cooldown>, ReadStorage<'a, BeenOnscreen>,
        Read<'a, LazyUpdate>, Read<'a, GameTime>, Read<'a, PlayerPositions>,
    );

    fn run(&mut self, (entities, pos, fires, mut cooldown, onscreen, updater, time, player_positions): Self::SystemData) {
        let mut rng = rand::thread_rng();

        for (pos, fires, cooldown, _) in (&pos, &fires, &mut cooldown, &onscreen).join() {
            if cooldown.is_ready(time.total_time) {
                match fires.method {
                    FiringMethod::AtPlayer(total, spread) => {
                        let player = player_positions.random(&mut rng);

                        // Get the rotation to the player
                        let rotation = (player.y - pos.0.y).atan2(player.x - pos.0.x);

                        for i in 0 .. total {
                            let mid_point = (total - 1) as f32 / 2.0;
                            let rotation_difference = spread * (mid_point - i as f32) / total as f32;

                            let rotation = rotation + rotation_difference;
                            build_bullet(&entities, &updater, pos.0, fires.image, Vector2::new(rotation.cos() * fires.speed, rotation.sin() * fires.speed), true);
                        }
                    }
                }
            }
        }
    }
}

pub struct Collisions;

impl<'a> System<'a> for Collisions {
    type SystemData = (
        Entities<'a>, Read<'a, LazyUpdate>, Read<'a, GameTime>,
        ReadStorage<'a, Position>, ReadStorage<'a, Friendly>, ReadStorage<'a, Enemy>, ReadStorage<'a, Hitbox>, ReadStorage<'a, FrozenUntil>,
        WriteStorage<'a, Health>, WriteStorage<'a, Invulnerability>,
    );

    fn run(&mut self, (entities, updater, time, pos, friendly, enemy, hitbox, frozen, mut health, mut invul): Self::SystemData) {
        let mut rng = rand::thread_rng();
        
        (&entities, &pos, &hitbox, &friendly).join()
            .flat_map(|friendly| {
                (&entities, &pos, &hitbox, !&frozen, &enemy).join()
                    .map(move |enemy| (friendly, enemy))
            })
            .for_each(|((f_entity, f_pos, f_hitbox, _), (e_entity, e_pos, e_hitbox, _, _))| {
                if let Some(mut hit_pos) = is_touching(f_pos.0, f_hitbox.0, e_pos.0, e_hitbox.0) {

                    let (player_triggered_invul, _) = damage_entity(f_entity, &entities, &mut health, &mut invul, time.total_time);
                    if player_triggered_invul {
                        let (_, enemy_dead) = damage_entity(e_entity, &entities, &mut health, &mut invul, time.total_time);

                        hit_pos.x += rng.gen_range(-5.0, 5.0);
                        hit_pos.y += rng.gen_range(-5.0, 5.0);
            
                        build_explosion(&updater, &entities, hit_pos, time.total_time);

                        if enemy_dead && rng.gen_range(0.0, 1.0) > 0.6 {
                            let (value, image) = if rng.gen_range(0.0, 1.0) > 0.9 { (5, GraphicsImage::BigOrb) } else { (1, GraphicsImage::Orb) };
                            updater.create_entity(&entities)
                                .with(Position(hit_pos))
                                .with(PowerOrb(value))
                                .with(Movement::Falling { speed: 0.0, down: true })
                                .with(Image::from(image))
                                .with(Hitbox(Vector2::new(50.0, 50.0)))
                                .with(DieOffscreen)
                                .build();
                        }
                    }
                }
            });
    }
}

fn damage_entity(entity: Entity, entities: &Entities, health: &mut WriteStorage<Health>, invul: &mut WriteStorage<Invulnerability>, time: f32) -> (bool, bool) {
    let (mut triggered_invul, mut dead) = (false, false);
    
    if let Some(health) = health.get_mut(entity) {
        triggered_invul = invul.get_mut(entity).map(|invul| invul.can_damage(time)).unwrap_or(true);

        if triggered_invul {
            health.0 = health.0.saturating_sub(1);

            if health.0 == 0 {
                dead = true;
                entities.delete(entity).unwrap();
            }
        }
    }

    (triggered_invul, dead)
}

pub struct ExpandBombs;

impl<'a> System<'a> for ExpandBombs {
    type SystemData = (Entities<'a>, Read<'a, specs::world::LazyUpdate>, Read<'a, GameTime>, WriteStorage<'a, Circle>, ReadStorage<'a, Position>, ReadStorage<'a, CollidesWithBomb>);

    fn run(&mut self, (entities, updater, time, mut circle, position, collides): Self::SystemData) {
        for (entity, mut circle, circle_pos) in (&entities, &mut circle, &position).join() {
            circle.radius += 8.0;
            
            if circle.radius.powi(2) >= Vector2::new(WIDTH, HEIGHT).magnitude2() {
                entities.delete(entity).unwrap();
            }

            for (entity, pos, _) in (&entities, &position, &collides).join() {
                if pos.0.distance2(circle_pos.0) <= circle.radius.powi(2) {
                    entities.delete(entity).unwrap();

                    build_explosion(&updater, &entities, pos.0, time.total_time);
                }
            }
        }
    }
}

fn build_explosion(updater: &specs::world::LazyUpdate, entities: &Entities, pos: Vector2<f32>, time: f32) {
    updater.create_entity(&entities)
        .with(Position(pos))
        .with(Explosion(time))
        .build();
}
