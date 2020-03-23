use specs::prelude::*;
use crate::{components, graphics, WIDTH, HEIGHT};
use components::Curve;
use cgmath::Vector2;

pub fn stage_one(mut world: &mut World) {
    let middle = Vector2::new(WIDTH / 2.0, HEIGHT / 2.0);

    world.create_entity()
            .with(components::Position(middle))
            .with(components::Image::from(graphics::Image::NightSky))
            .with(components::BackgroundLayer)
            .build();

    world.create_entity()
        .with(components::Position(middle))
        .with(components::Image::from(graphics::Image::Clouds))
        .with(components::Movement::Linear(Vector2::new(0.0, 1.0)))
        .with(components::BackgroundLayer)
        .build();

    world.create_entity()
        .with(components::Position(middle + Vector2::new(0.0, 1920.0)))
        .with(components::Image::from(graphics::Image::Clouds))
        .with(components::Movement::Linear(Vector2::new(0.0, 1.0)))
        .with(components::BackgroundLayer)
        .build();

    create_players(&mut world, false);

    for start in float_iter(10.0, 60.0, 0.25) {
        create_bat_with_curve(&mut world, Curve::horizontal(100.0, 300.0, true, 2.5), start);
        create_bat_with_curve(&mut world, Curve::horizontal(150.0, 350.0, true, 2.5), start);
    }

    for start in float_iter(30.0, 100.0, 0.25) {
        create_bat_with_curve(&mut world, Curve::horizontal(200.0, 400.0, false, 2.5), start);
        create_bat_with_curve(&mut world, Curve::horizontal(250.0, 450.0, false, 2.5), start);
    }

    for start in float_iter(12.0, 17.0, 0.25) {
        create_bat_with_curve(&mut world, Curve::vertical(0.25, 0.5, 2.5), start);
        create_bat_with_curve(&mut world, Curve::vertical(0.5, 0.75, 2.5), start);
        create_bat_with_curve(&mut world, Curve::vertical(0.75, 0.25, 2.5), start);
    }

    for start in float_iter(15.0, 20.0, 0.5) {
        create_bat_with_curve(&mut world, Curve::horizontal(400.0, 600.0, true, 2.5), start)
    }

    for x in [0.25, 0.5, 0.75].iter() {
        world.create_entity()
            .with(components::Movement::FiringMove(2.5, 34.0, 100.0))
            .with(components::Position(Vector2::new(x * WIDTH, -50.0)))
            .with(components::Image::from(graphics::Image::Gargoyle))
            .with(components::DieOffscreen)
            .with(components::FrozenUntil(23.0))
            .with(components::FiresBullets {
                image: components::Image::from(graphics::Image::RockBullet),
                speed: 2.5,
                method: components::FiringMethod::AtPlayer(3, 1.0),
            })
            .with(components::Cooldown::new(1.0))
            .build();
    }
}

fn create_bat_with_curve(mut world: &mut World, curve: components::Curve, start: f32) {
    entity_with_curve(&mut world, curve)
        .with(components::Image::from(graphics::Image::Bat))
        .with(components::FrozenUntil(start))
        .with(components::DieOffscreen)
        .with(components::Hitbox(Vector2::new(25.0, 20.0)))
        .with(components::Enemy)
        .with(components::Health(4))
        .build();
}

fn entity_with_curve(world: &mut World, curve: components::Curve) -> EntityBuilder {
    world.create_entity()
        .with(components::Movement::FollowCurve(curve.clone()))
        .with(components::Position(curve.b))
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

fn create_players(mut world: &mut World, two_players: bool) {
    let middle = Vector2::new(WIDTH / 2.0, HEIGHT / 2.0);

    if two_players {
        let (player_one, player_two) = components::Controllable::two_players();
        create_player(world, player_one, middle);
        create_player(world, player_two, middle);
    } else {
        create_player(world, components::Controllable::one_player(), middle);
    }
}

fn create_player(mut world: &mut World, controls: components::Controllable, position: Vector2<f32>) {
    world.create_entity()
            .with(components::Position(position))
            .with(components::Image::from(graphics::Image::Player))
            .with(controls)
            .with(components::Cooldown::new(0.075))
            .with(components::Hitbox(Vector2::new(10.0, 10.0)))
            .with(components::Friendly)
            .with(components::Health(10))
            .with(components::Invulnerability::new())
            .build();
}
