use specs::prelude::*;
use cgmath::Vector2;
use rand::Rng;
use crate::{resources::*, components::*};

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
