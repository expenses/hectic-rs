use specs::prelude::*;
use specs::world::LazyBuilder;
use crate::{components::*, graphics, WIDTH, HEIGHT, ZERO, MIDDLE};
use cgmath::Vector2;
use rand::Rng;

struct EntityBuilder<'a> {
    entities: &'a Entities<'a>,
    updater: &'a LazyUpdate
}

impl EntityBuilder<'_> {
    fn create_entity(&self) -> LazyBuilder<'_> {
        self.updater.create_entity(self.entities)
    } 
}

fn clear(builder: &EntityBuilder) {
    builder.entities.join().for_each(|entity| builder.entities.delete(entity).unwrap());
}

pub fn stage_one(entities: &Entities, updater: &LazyUpdate, multiplayer: bool, time: &mut f32) {
    let mut rng = rand::thread_rng();
    let builder = &EntityBuilder { entities, updater };

    *time = 0.0;
    clear(builder);
    create_background(builder, graphics::Image::NightSky, ZERO, ZERO, 0);
    create_background(builder, graphics::Image::Clouds, ZERO, Vector2::new(0.0, 1.0), 1);
    create_background(builder, graphics::Image::Clouds, Vector2::new(0.0, 1920.0), Vector2::new(0.0, 1.0), 1);
    create_title(builder, "Stage\nOne");
    create_players(builder, multiplayer);

    for start in float_iter(1.0, 6.0, 0.25) {
        bat_with_curve(builder, FollowCurve::horizontal(100.0, 300.0, true, 2.5), start);
        bat_with_curve(builder, FollowCurve::horizontal(150.0, 350.0, true, 2.5), start);
    }

    for start in float_iter(3.0, 10.0, 0.25) {
        bat_with_curve(builder, FollowCurve::horizontal(200.0, 400.0, false, 2.5), start);
        bat_with_curve(builder, FollowCurve::horizontal(250.0, 450.0, false, 2.5), start);
    }

    for start in float_iter(12.0, 17.0, 0.25) {
        bat_with_curve(builder, FollowCurve::vertical(0.25, 0.5, 2.5), start);
        bat_with_curve(builder, FollowCurve::vertical(0.5, 0.75, 2.5), start);
        bat_with_curve(builder, FollowCurve::vertical(0.75, 0.25, 2.5), start);
    }

    for start in float_iter(15.0, 20.0, 0.5) {
        bat_with_curve(builder, FollowCurve::horizontal(400.0, 600.0, true, 2.5), start)
    }

    let rock_bullet = BulletSetup { image: Image::from(graphics::Image::RockBullet), speed: 2.8, colour: None };

    for x in [0.25, 0.5, 0.75].iter() {
        let start = 24.0;

        enemy(
            builder,
            Vector2::new(x * WIDTH, -50.0),
            start,
            15,
            graphics::Image::Gargoyle,
            Vector2::new(45.0, 25.0),
        )
            .with(FiringMove { speed: 2.5, return_time: start + 10.0, stop_time: start + 1.0 })
            .with(FiresBullets::AtPlayer {
                num_bullets: 3,
                spread: 1.0,
                cooldown: Cooldown::ready_at(1.0, start + 1.0 + rng.gen_range(-0.25, 0.5)),
                setup: rock_bullet
            })
            .build();
    }

    for x in [0.375, 0.625].iter() {
        let start = 28.0;

        enemy(
            builder,
            Vector2::new(x * WIDTH, -50.0),
            start,
            15,
            graphics::Image::Gargoyle,
            Vector2::new(45.0, 25.0),
        )
            .with(FiringMove { speed: 2.5, return_time: start + 10.0, stop_time: start + 1.0 })
            .with(FiresBullets::AtPlayer {
                num_bullets: 3,
                spread: 1.0,
                cooldown: Cooldown::ready_at(1.0, start + 1.0 + rng.gen_range(-0.25, 0.5)),
                setup: rock_bullet
            })
            .build();
    }

    for start in float_iter(25.0, 33.0, 0.25) {
        bat_with_curve(builder, FollowCurve::circular(200.0, 1000.0, 2.5), start);
    }

    for start in float_iter(35.0, 50.0, 0.25) {
        hell_bat(builder, start, Vector2::new(rng.gen_range(0.0, WIDTH), -50.0));
    }

    for start in float_iter(45.0, 50.0, 1.0) {
        enemy_with_curve(
            builder,
            FollowCurve::horizontal(100.0, 300.0, true, 2.5),
            start,
            15,
            graphics::Image::Gargoyle,
            Vector2::new(45.0, 25.0),
        )
            .with(FiresBullets::AtPlayer {
                num_bullets: 1,
                spread: 0.0,
                cooldown: Cooldown::ready_at(1.0, start + rng.gen_range(0.0, 0.5)),
                setup: rock_bullet
            })
            .build();
    }

    boss_one(builder, 55.0);
}

fn hell_bat(builder: &EntityBuilder, start: f32, position: Vector2<f32>) {
    builder.create_entity()
        .with(Position(position))
        .with(FrozenUntil(start))
        .with(DieOffscreen)
        .with(Enemy)
        .with(Health(12))
        .with(Image::from(graphics::Image::HellBat))
        .with(Hitbox(Vector2::new(25.0, 20.0)))
        .with(TargetPlayer(2.5))
        .build();
}

fn bat_with_curve(builder: &EntityBuilder, curve: FollowCurve, start: f32) {
    enemy_with_curve(builder, curve, start, 4, graphics::Image::Bat, Vector2::new(25.0, 20.0)).build();
}

fn enemy<'a>(builder: &'a EntityBuilder, position: Vector2<f32>, start: f32, health: u32, image: graphics::Image, hitbox: Vector2<f32>) -> LazyBuilder<'a> {
    builder.create_entity()
        .with(Position(position))
        .with(FrozenUntil(start))
        .with(DieOffscreen)
        .with(Enemy)
        .with(Health(health))
        .with(Image::from(image))
        .with(Hitbox(hitbox))
}

fn enemy_with_curve<'a>(builder: &'a EntityBuilder, curve: FollowCurve, start: f32, health: u32, image: graphics::Image, hitbox: Vector2<f32>) -> LazyBuilder<'a> {
    enemy(
        builder,
        curve.start(),
        start,
        health,
        image,
        hitbox,
    ).with(curve)
}

fn float_iter(start: f32, end: f32, step: f32) -> impl Iterator<Item = f32> {
    std::iter::repeat(())
        .scan(start, move |value, _| {
            let item = *value;
            *value += step;
            Some(item)
        })
        .take_while(move |item| *item < end)
}

fn create_players(builder: &EntityBuilder, two_players: bool) {
    if two_players {
        let offset = Vector2::new(20.0, 0.0);
        create_player(builder, Player::One, MIDDLE - offset);
        create_player(builder, Player::Two, MIDDLE + offset);
    } else {
        create_player(builder, Player::Single, MIDDLE);
    }
}

fn create_player(builder: &EntityBuilder, player: Player, position: Vector2<f32>) {
    builder.create_entity()
            .with(Position(position))
            .with(Image::from(graphics::Image::Player))
            .with(player)
            .with(Cooldown::new(0.075))
            .with(Hitbox(Vector2::new(10.0, 10.0)))
            .with(Friendly)
            .with(Health(3))
            .with(Invulnerability::new())
            .with(PowerBar(0))
            .build();
}

fn create_title(builder: &EntityBuilder, text: &'static str) {
    builder.create_entity()
        .with(Text::title(text))
        .with(Position(Vector2::new(WIDTH / 2.0, 40.0)))
        .with(Falling { speed: 0.0, down: false })
        .build();
}

fn create_background(builder: &EntityBuilder, image: graphics::Image, position: Vector2<f32>, velocity: Vector2<f32>, depth: u32) {
    builder.create_entity()
        .with(Position(MIDDLE + position))
        .with(Image::from(image))
        .with(Velocity(velocity))
        .with(BackgroundLayer { depth })
        .build();
}

pub fn stage_two(entities: &Entities, updater: &LazyUpdate, multiplayer: bool, time: &mut f32) {
    let mut rng = rand::thread_rng();
    let builder = &EntityBuilder { entities, updater };

    *time = 0.0;
    clear(builder);
    create_background(builder, graphics::Image::Graveyard, ZERO, Vector2::new(0.0, 0.5), 0);
    create_background(builder, graphics::Image::Graveyard, Vector2::new(0.0, 1440.0), Vector2::new(0.0, 0.5), 0);
    create_background(builder, graphics::Image::Fog, ZERO, Vector2::new(0.0, 0.5), 1);
    create_background(builder, graphics::Image::Fog, Vector2::new(0.0, 1920.0), Vector2::new(0.0, 0.5), 1);
    create_background(builder, graphics::Image::Darkness, ZERO, ZERO, 2);
    create_title(builder, "Stage\nTwo");
    create_players(builder, multiplayer);

    let spectre_speed = 10.0 / 3.0;

    for start in float_iter(5.0, 20.0, 0.5) {
        let setup = BulletSetup {
            image: Image::from(graphics::Image::DarkBullet),
            speed: spectre_speed,
            colour: None
        };

        enemy_with_curve(
            builder,
            FollowCurve::horizontal(rng.gen_range(0.0, HEIGHT / 2.0), rng.gen_range(0.0, HEIGHT / 2.0), true, spectre_speed),
            start, 8, graphics::Image::Spectre, Vector2::new(30.0, 30.0),
        )
            .with(FiresBullets::AtPlayer { num_bullets: 1, spread: 0.0, cooldown: Cooldown::ready_at(1.0, rng.gen_range(start, start + 1.0)), setup })
            .build();
    }

    for start in float_iter(25.0, 45.0, 0.5) {
        flying_skull(builder, start, Vector2::new(rng.gen_range(0.0, WIDTH), -25.0));
        
        if start >= 30.0 {
            flying_skull(builder, start, Vector2::new(rng.gen_range(0.0, WIDTH), -25.0));
            flying_skull(builder, start, Vector2::new(-25.0, rng.gen_range(0.0, HEIGHT / 2.0)));
            flying_skull(builder, start, Vector2::new(WIDTH + 25.0, rng.gen_range(0.0, HEIGHT / 2.0)));
        }
    }

    boss_two(builder, 50.0);
}

fn flying_skull(builder: &EntityBuilder, start: f32, position: Vector2<f32>) {
    builder.create_entity()
        .with(Position(position))
        .with(FrozenUntil(start))
        .with(DieOffscreen)
        .with(Enemy)
        .with(Health(4))
        .with(Image::from(graphics::Image::FlyingSkull))
        .with(Hitbox(Vector2::new(25.0, 25.0)))
        .with(TargetPlayer(10.0 / 3.0))
        .build();
}

fn boss_one(builder: &EntityBuilder, start: f32) {
    let speed = 10.0 / 3.0;
    let orange_bullet = BulletSetup {
        image: Image::from(graphics::Image::Sword),
        speed,
        colour: None
    };

    builder.create_entity()
        .with(Position(Vector2::new(WIDTH / 2.0, -50.0)))
        .with(FrozenUntil(start))
        .with(DieOffscreen)
        .with(Enemy)
        .with(Health(300))
        .with(Image::from(graphics::Image::BossOne))
        .with(Hitbox(Vector2::new(30.0, 40.0)))
        .with(Boss {
            max_health: 300,
            current_move: 0,
            move_timer: 0.0,
            moves: vec![
                BossMove {
                    position: Vector2::new(100.0, 100.0),
                    fires: FiresBullets::Multiple(vec![
                        FiresBullets::Arc { initial_rotation: 0.0, spread: 2.0, number_to_fire: 20, fired_at_once: 1, fired_so_far: 0, cooldown: Cooldown::new(0.05), setup: orange_bullet },
                        FiresBullets::Arc { initial_rotation: 2.0, spread: -2.0, number_to_fire: 20, fired_at_once: 1, fired_so_far: 0, cooldown: Cooldown::new(0.05), setup: orange_bullet }
                    ]),
                    duration: 4.0,
                },
                BossMove {
                    position: Vector2::new(150.0, 150.0),
                    fires: FiresBullets::Multiple(vec![
                        FiresBullets::AtPlayer { num_bullets: 3, spread: 1.0, cooldown: Cooldown::new(0.75), setup: orange_bullet },
                        FiresBullets::Circle { sides: 4, rotation_per_fire: 0.5, rotation: 0.0, cooldown: Cooldown::new(0.1), setup: orange_bullet }
                    ]),
                    duration: 6.0,
                },
                BossMove {
                    position: Vector2::new(WIDTH / 2.0, 100.0),
                    fires: FiresBullets::Circle { sides: 6, rotation_per_fire: 0.2, rotation: 0.0, cooldown: Cooldown::new(0.1), setup: orange_bullet },
                    duration: 6.0,
                },
                BossMove {
                    position: Vector2::new(400.0, 200.0),
                    fires: FiresBullets::AtPlayer { num_bullets: 5, spread: 0.5, cooldown: Cooldown::new(0.25), setup: orange_bullet },
                    duration: 2.0,
                },
            ]
        })
        .build();
}

fn boss_two(builder: &EntityBuilder, start: f32) {
    let speed = 10.0 / 3.0;
    let dark_bullet = BulletSetup {
        image: Image::from(graphics::Image::DarkBullet),
        speed,
        colour: None
    };
    let purple_bullet = BulletSetup {
        image: Image::from(graphics::Image::ColouredBullet),
        speed,
        colour: Some(ColourBullets::Purple)
    };

    let pi = std::f32::consts::PI;


    builder.create_entity()
        .with(Position(Vector2::new(WIDTH / 2.0, -50.0)))
        .with(FrozenUntil(start))
        .with(DieOffscreen)
        .with(Enemy)
        .with(Health(400))
        .with(Image::from(graphics::Image::BossTwo))
        .with(Hitbox(Vector2::new(30.0, 40.0)))
        .with(Boss {
            max_health: 400,
            current_move: 0,
            move_timer: 0.0,
            moves: vec![
                BossMove {
                    position: Vector2::new(WIDTH / 2.0, 150.0),
                    fires: FiresBullets::Arc { initial_rotation: pi / 2.0, spread: pi * 2.0, number_to_fire: 100, fired_at_once: 2, fired_so_far: 0, cooldown: Cooldown::new(0.015), setup: dark_bullet },
                    duration: 3.0,
                },
                BossMove {
                    position: Vector2::new(WIDTH / 2.0 - 50.0, 160.0),
                    fires: FiresBullets::Arc { initial_rotation: pi / 2.0, spread: -pi * 2.0, number_to_fire: 100, fired_at_once: 2, fired_so_far: 0, cooldown: Cooldown::new(0.015), setup: dark_bullet },
                    duration: 3.0,
                },
                BossMove {
                    position: Vector2::new(WIDTH / 2.0 + 50.0, 170.0),
                    duration: 5.0,
                    fires: FiresBullets::Multiple(vec![
                        FiresBullets::Arc { initial_rotation: 0.0, spread: 2.0 * pi, number_to_fire: 101, fired_at_once: 2, fired_so_far: 0, cooldown: Cooldown::new(0.03), setup: dark_bullet },
                        FiresBullets::Arc { initial_rotation: 2.0 * pi, spread: 2.0 * -pi, number_to_fire: 101, fired_at_once: 2, fired_so_far: 0, cooldown: Cooldown::new(0.03), setup: dark_bullet }
                    ])
                },
                /*BossMove {
                    position: Vector2::new(100.0, 100.0),
                    fires: FiresBullets::Multiple(vec![
                        FiresBullets::Arc { initial_rotation: pi / 2.0, spread: -pi * 2.0, number_to_fire: 100, fired_at_once: 2, fired_so_far: 0, cooldown: Cooldown::new(0.015), setup: purple_bullet },
                        FiresBullets::AtPlayer { num_bullets: 3, spread: 0.1, cooldown: Cooldown::new(0.2), setup: dark_bullet }
                    ]),
                    duration: 5.0,
                },*/
                BossMove {
                    position: Vector2::new(100.0, 100.0),
                    fires: FiresBullets::Arc { initial_rotation: pi / 2.0, spread: 10.0 * -pi * 2.0, number_to_fire: 777, fired_at_once: 1, fired_so_far: 0, cooldown: Cooldown::new(0.015), setup: purple_bullet },
                    duration: 10.0,
                },
                
            ]
        })
        .build();
}
