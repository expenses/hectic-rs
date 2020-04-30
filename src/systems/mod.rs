use specs::prelude::*;

use crate::components::*;
use crate::resources::*;

use cgmath::Vector2;

use crate::graphics::Image as GraphicsImage;

const WIDTH: f32 = 480.0;
const HEIGHT: f32 = 640.0;
const PLAYER_SPEED: f32 = 250.0 / 60.0;
const PLAYER_BULLET_SPEED: f32 = 500.0 / 60.0;

mod rendering;
mod bullets;

pub use rendering::*;
pub use bullets::*;

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

pub struct TogglePaused;

impl<'a> System<'a> for TogglePaused {
    type SystemData = (Write<'a, ControlsState>, Write<'a, Mode>, Write<'a, Menu>);

    fn run(&mut self, (mut ctrl_state, mut mode, mut menu): Self::SystemData) {
        if ctrl_state.pause.pressed {
            *mode = match *mode {
                Mode::Playing => {
                    *menu = Menu {
                        title: "Paused".into(),
                        items: vec!["Resume".into(), "Main Menu".into()],
                        selected: 0
                    };
    
                    Mode::Paused
                },
                Mode::Paused => Mode::Playing,
                _ => *mode
            };
            ctrl_state.pause.pressed = false;
        }
    }
}

pub struct PauseMenu;

impl<'a> System<'a> for PauseMenu {
    type SystemData = (Write<'a, ControlsState>, Write<'a, Mode>, Write<'a, Menu>, ReadStorage<'a, Player>);

    fn run(&mut self, (mut ctrl_state, mut mode, mut menu, player): Self::SystemData) {
        for player in (&player).join() {
            let player_ctrl_state = ctrl_state.get_mut(*player);

            if player_ctrl_state.down.pressed {
                menu.rotate_down();
                player_ctrl_state.down.pressed = false;
            }

            if player_ctrl_state.up.pressed {
                menu.rotate_up();
                player_ctrl_state.up.pressed = false;
            }

            if player_ctrl_state.fire.pressed {
                *mode = match menu.selected {
                    0 => Mode::Playing,
                    1 => Mode::MainMenu,
                    _ => unreachable!()
                };

                player_ctrl_state.fire.pressed = false;
            }
        }
    }
}

pub struct Control;

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
