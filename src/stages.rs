use specs::prelude::*;
use crate::{components, graphics, WIDTH, HEIGHT};
use components::Curve;
use cgmath::Vector2;
use rand::Rng;

pub const ZERO: Vector2<f32> = Vector2::new(0.0, 0.0);
pub const MIDDLE: Vector2<f32> = Vector2::new(WIDTH / 2.0, HEIGHT / 2.0);

pub fn stage_one(world: &mut World) {
    create_background(world, graphics::Image::NightSky, ZERO, ZERO);
    create_background(world, graphics::Image::Clouds, ZERO, Vector2::new(0.0, 1.0));
    create_background(world, graphics::Image::Clouds, Vector2::new(0.0, 1920.0), Vector2::new(0.0, 1.0));
    create_title(world, "Stage\nOne");
    create_players(world, false);

    for start in float_iter(10.0, 60.0, 0.25) {
        bat_with_curve(world, Curve::horizontal(100.0, 300.0, true, 2.5), start);
        bat_with_curve(world, Curve::horizontal(150.0, 350.0, true, 2.5), start);
    }

    for start in float_iter(30.0, 100.0, 0.25) {
        bat_with_curve(world, Curve::horizontal(200.0, 400.0, false, 2.5), start);
        bat_with_curve(world, Curve::horizontal(250.0, 450.0, false, 2.5), start);
    }

    for start in float_iter(12.0, 17.0, 0.25) {
        bat_with_curve(world, Curve::vertical(0.25, 0.5, 2.5), start);
        bat_with_curve(world, Curve::vertical(0.5, 0.75, 2.5), start);
        bat_with_curve(world, Curve::vertical(0.75, 0.25, 2.5), start);
    }

    for start in float_iter(15.0, 20.0, 0.5) {
        bat_with_curve(world, Curve::horizontal(400.0, 600.0, true, 2.5), start)
    }

    for x in [0.25, 0.5, 0.75].iter() {
        enemy(
            world,
            Vector2::new(x * WIDTH, -50.0),
            components::Movement::FiringMove(2.5, 34.0, 100.0),
            23.0,
            15,
            graphics::Image::Gargoyle,
            Vector2::new(45.0, 25.0),
        )
            .with(components::FiresBullets {
                image: components::Image::from(graphics::Image::RockBullet),
                speed: 2.5,
                method: components::FiringMethod::AtPlayer(3, 1.0),
            })
            .with(components::Cooldown::new(1.0))
            .build();
    }
}

fn bat_with_curve(world: &mut World, curve: components::Curve, start: f32) {
    enemy_with_curve(world, curve, start, 4, graphics::Image::Bat, Vector2::new(25.0, 20.0)).build();
}

fn enemy(world: &mut World, position: Vector2<f32>, movement: components::Movement, start: f32, health: u32, image: graphics::Image, hitbox: Vector2<f32>) -> EntityBuilder {
    world.create_entity()
        .with(movement)
        .with(components::Position(position))
        .with(components::FrozenUntil(start))
        .with(components::DieOffscreen)
        .with(components::Enemy)
        .with(components::Health(health))
        .with(components::Image::from(image))
        .with(components::Hitbox(hitbox))
}

fn enemy_with_curve(world: &mut World, curve: components::Curve, start: f32, health: u32, image: graphics::Image, hitbox: Vector2<f32>) -> EntityBuilder {
    enemy(
        world,
        curve.b,
        components::Movement::FollowCurve(curve),
        start,
        health,
        image,
        hitbox
    )
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

fn create_players(world: &mut World, two_players: bool) {
    if two_players {
        let offset = Vector2::new(20.0, 0.0);
        create_player(world, components::Player::One, MIDDLE - offset);
        create_player(world, components::Player::Two, MIDDLE + offset);
    } else {
        create_player(world, components::Player::Single, MIDDLE);
    }
}

fn create_player(world: &mut World, player: components::Player, position: Vector2<f32>) {
    world.create_entity()
            .with(components::Position(position))
            .with(components::Image::from(graphics::Image::Player))
            .with(player)
            .with(components::Cooldown::new(0.075))
            .with(components::Hitbox(Vector2::new(10.0, 10.0)))
            .with(components::Friendly)
            .with(components::Health(2))
            .with(components::Invulnerability::new())
            .build();
}

fn create_title(world: &mut World, text: &'static str) {
    world.create_entity()
        .with(components::Text::title(text))
        .with(components::Position(Vector2::new(WIDTH / 2.0, 40.0)))
        .with(components::Movement::Falling(0.0))
        .build();
}

fn create_background(world: &mut World, image: graphics::Image, position: Vector2<f32>, movement: Vector2<f32>) {
    world.create_entity()
        .with(components::Position(MIDDLE + position))
        .with(components::Image::from(image))
        .with(components::Movement::Linear(movement))
        .with(components::BackgroundLayer)
        .build();
}

pub fn stage_two(world: &mut World) {
    let mut rng = rand::thread_rng();

    create_background(world, graphics::Image::Graveyard, ZERO, Vector2::new(0.0, 0.5));
    create_background(world, graphics::Image::Graveyard, Vector2::new(0.0, 1440.0), Vector2::new(0.0, 0.5));
    create_background(world, graphics::Image::Fog, ZERO, Vector2::new(0.0, 0.5));
    create_background(world, graphics::Image::Fog, Vector2::new(0.0, 1920.0), Vector2::new(0.0, 0.5));
    create_background(world, graphics::Image::Darkness, ZERO, ZERO);
    create_title(world, "Stage\nTwo");
    create_players(world, false);

    let spectre_speed = 10.0 / 3.0;

    for start in float_iter(5.0, 20.0, 0.5) {
        enemy_with_curve(
            world,
            Curve::horizontal(rng.gen_range(0.0, HEIGHT / 2.0), rng.gen_range(0.0, HEIGHT / 2.0), true, spectre_speed),
            start, 8, graphics::Image::Spectre, Vector2::new(30.0, 30.0),
        )
            .with(components::FiresBullets {
                image: components::Image::from(graphics::Image::DarkBullet),
                speed: spectre_speed,
                method: components::FiringMethod::AtPlayer(1, 0.0),
            })
            .with(components::Cooldown::new(1.0))
            .build();
    }

    for start in float_iter(25.0, 45.0, 0.5) {
        flying_skull(world, start, Vector2::new(rng.gen_range(0.0, WIDTH), -25.0));
        
        if start >= 30.0 {
            flying_skull(world, start, Vector2::new(rng.gen_range(0.0, WIDTH), -25.0));
            flying_skull(world, start, Vector2::new(-25.0, rng.gen_range(0.0, HEIGHT / 2.0)));
            flying_skull(world, start, Vector2::new(WIDTH + 25.0, rng.gen_range(0.0, HEIGHT / 2.0)));
        }
    }
}

fn flying_skull(world: &mut World, start: f32, position: Vector2<f32>) {
    world.create_entity()
        .with(components::Position(position))
        .with(components::FrozenUntil(start))
        .with(components::DieOffscreen)
        .with(components::Enemy)
        .with(components::Health(4))
        .with(components::Image::from(graphics::Image::FlyingSkull))
        .with(components::Hitbox(Vector2::new(25.0, 25.0)))
        .with(components::TargetPlayer(10.0 / 3.0))
        .build();
}

/*
Stage stageTwo() {
    // Add a bunch of spectres

    BulletFactory darkBullets = new BulletFactory(resources.darkBullet, 200);
    for (float s = 5; s < 20; s += 0.5) {
        stage.add(s, new Spectre(new HorizontalCurve(random(0, HEIGHT/2), random(0, HEIGHT/2), true), new AtPlayer(1, 0, 1, darkBullets)));
    }

    // Lots of flying skulls on all sides! AAahh!!

    for (float s = 25; s < 45; s += 0.5) {
        stage.add(s, new FlyingSkull(new TargetPlayer(random(0, WIDTH), -25)));

        if (s >= 30) {
            stage.add(s, new FlyingSkull(new TargetPlayer(random(0, WIDTH), -25)));
            stage.add(s, new FlyingSkull(new TargetPlayer(-25, random(0, HEIGHT/2))));
            stage.add(s, new FlyingSkull(new TargetPlayer(WIDTH + 25, random(0, HEIGHT/2))));
        }
    }

    // And the boss again

    stage.add(50, new BossTwo());

    return stage;
}
*/
