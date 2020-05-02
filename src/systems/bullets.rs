use specs::prelude::*;
use cgmath::{Vector2, MetricSpace, InnerSpace};
use rand::{Rng, rngs::ThreadRng};
use crate::{WIDTH, HEIGHT, resources::*, components::*, graphics::Image as GraphicsImage};
use super::{is_touching, build_bullet};

pub struct FireBullets;

impl<'a> System<'a> for FireBullets {
    type SystemData = (
        Entities<'a>, ReadStorage<'a, Position>, WriteStorage<'a, FiresBullets>, ReadStorage<'a, BeenOnscreen>,
        Read<'a, LazyUpdate>, Read<'a, GameTime>, Read<'a, PlayerPositions>,
    );

    fn run(&mut self, (entities, pos, mut fires, onscreen, updater, time, player_positions): Self::SystemData) {
        let mut rng = rand::thread_rng();

        for (pos, mut fires, _) in (&pos, &mut fires, &onscreen).join() {
            handle_fires_bullets(&entities, &updater, &mut fires, time.total_time, &player_positions, &mut rng, pos.0);
        }
    }
}

pub fn handle_fires_bullets(
    entities: &Entities, updater: &LazyUpdate, fires: &mut FiresBullets,
    total_time: f32, player_positions: &PlayerPositions, rng: &mut ThreadRng,
    pos: Vector2<f32>,
) {
    match fires {
        FiresBullets::AtPlayer { num_bullets, spread, cooldown, setup } => if cooldown.is_ready(total_time) {
            let player = player_positions.random(rng);

            // Get the rotation to the player
            let rotation = (player.y - pos.y).atan2(player.x - pos.x);

            for i in 0 .. *num_bullets {
                let mid_point = (*num_bullets - 1) as f32 / 2.0;
                let rotation_difference = *spread * (mid_point - i as f32) / *num_bullets as f32;

                let rotation = rotation + rotation_difference;
                build_bullet(entities, updater, pos, setup.image, Vector2::new(rotation.cos(), rotation.sin()) * setup.speed, true, setup.colour);
            }
        },
        FiresBullets::Circle { sides, rotation, rotation_per_fire, cooldown, setup } => if cooldown.is_ready(total_time) {
            for side in 0 .. *sides {
                let rotation = (side as f32 / *sides as f32) * std::f32::consts::PI * 2.0 + *rotation;
                build_bullet(entities, updater, pos, setup.image, Vector2::new(rotation.cos(), rotation.sin()) * setup.speed, true, setup.colour);
            }

            *rotation += *rotation_per_fire;
        },
        FiresBullets::Arc { initial_rotation, spread, fired_at_once, number_to_fire, fired_so_far, cooldown, setup } => if cooldown.is_ready(total_time) {
            for _ in 0 .. *fired_at_once {
                if *fired_so_far < *number_to_fire {
                    let rotation = *initial_rotation + *spread * (*fired_so_far as f32 / *number_to_fire as f32);
                    build_bullet(entities, updater, pos, setup.image, Vector2::new(rotation.cos(), rotation.sin()) * setup.speed, true, setup.colour);
                    *fired_so_far += 1;
                }
            }
        },
        FiresBullets::Multiple(vec) => for fires in vec { handle_fires_bullets(entities, updater, fires, total_time, player_positions, rng, pos); }
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
