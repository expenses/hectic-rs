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
    type SystemData = (ReadStorage<'a, Position>, ReadStorage<'a, Image>, Write<'a, Renderer>);

    fn run(&mut self, (pos, image, mut renderer): Self::SystemData) {
        for (pos, image) in (&pos, &image).join() {
            renderer.render_sprite(*image, pos.0);
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
                    if *return_time <= game_time.0 {
                        pos.0.y -= *speed;
                    } else {
                        pos.0.y = min(pos.0.y + *speed, *stop_y);
                    }
                }
            }
        }
    }
}

pub struct HandleKeypresses;

impl<'a> System<'a> for HandleKeypresses {
    type SystemData = (Write<'a, KeyPresses>, Write<'a, KeyboardState>);

    fn run(&mut self, (mut presses, mut kdb_state): Self::SystemData) {
        for (key, pressed) in presses.0.drain(..) {
            kdb_state.0.insert(key, pressed);
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
    type SystemData = (Read<'a, KeyboardState>, ReadStorage<'a, Controllable>, WriteStorage<'a, Position>, Write<'a, BulletSpawner>);

    fn run(&mut self, (kdb_state, controllable, mut position, mut spawner): Self::SystemData) {
        for (controls, mut pos) in (&controllable, &mut position).join() {
            if kdb_state.is_pressed(controls.left) {
                pos.0.x = max(pos.0.x - PLAYER_SPEED, 0.0);
            }

            if kdb_state.is_pressed(controls.right) {
                pos.0.x = min(pos.0.x + PLAYER_SPEED, WIDTH);
            }

            if kdb_state.is_pressed(controls.up) {
                pos.0.y = max(pos.0.y - PLAYER_SPEED, 0.0);
            }

            if kdb_state.is_pressed(controls.down) {
                pos.0.y = min(pos.0.y + PLAYER_SPEED, HEIGHT);
            }

            if kdb_state.is_pressed(controls.fire) {
                spawner.0.push(BulletToBeSpawned {
                    pos: pos.0,
                    image: Image::from(GraphicsImage::PlayerBullet),
                    velocity: Vector2::new(0.0, -PLAYER_BULLET_SPEED),
                });
            }
        }
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
    type SystemData = (ReadStorage<'a, Position>, ReadStorage<'a, Controllable>, WriteStorage<'a, FiresBullets>, Write<'a, BulletSpawner>, Read<'a, GameTime>);

    fn run(&mut self, (pos, controllable, mut fires, mut spawner, time): Self::SystemData) {
        let player_positions = (&pos, &controllable).join()
            .map(|(pos, _)| pos.0)
            .collect::<Vec<_>>();

        for (pos, mut fires) in (&pos, &mut fires).join() {
            if fires.last_fired + fires.cooldown <= time.0 {
                match fires.method {
                    FiringMethod::AtPlayer(total, spread) => {
                        let player = rand::thread_rng().gen_range(0, player_positions.len());
                        let player = player_positions[player];

                        // Get the rotation to the player
                        let rotation = (player.y - pos.0.y).atan2(player.x - pos.0.x);

                        for i in 0 .. total {
                            let mid_point = (total - 1) as f32 / 2.0;
                            let rotation_difference = spread * (mid_point - i as f32) / total as f32;

                            spawner.0.push(fires.bullet_to_be_spawned(pos.0, rotation + rotation_difference));
                        }
                    }
                }

                fires.last_fired = time.0;
            }
        }
    }
}

pub struct SpawnBullets;

impl<'a> System<'a> for SpawnBullets {
    type SystemData = (
        Entities<'a>, Write<'a, BulletSpawner>,
        WriteStorage<'a, Position>, WriteStorage<'a, Image>, WriteStorage<'a, Movement>, WriteStorage<'a, DieOffscreen>,
    );

    fn run(&mut self, (entities, mut spawner, mut pos, mut image, mut mov, mut dieoffscreen): Self::SystemData) {
        for bullet in spawner.0.drain(..) {
            entities.build_entity()
                .with(Position(bullet.pos), &mut pos)
                .with(bullet.image, &mut image)
                .with(Movement::Linear(bullet.velocity), &mut mov)
                .with(DieOffscreen, &mut dieoffscreen)
                .build();
        }
    }
}
