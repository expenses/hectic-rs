use specs::prelude::*;
use crate::{components, graphics, WIDTH, HEIGHT};
use components::Curve;
use cgmath::Vector2;

pub fn stage_one(mut world: &mut World) {
    let middle = Vector2::new(WIDTH/2.0, HEIGHT / 2.0);

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

    world.create_entity()
        .with(components::Position(middle))
        .with(components::Image::from(graphics::Image::Player))
        .with(components::Controllable)
        .build();
    
    for start in float_iter(1.0, 6.0, 0.25) {
        create_bat_with_curve(&mut world, Curve::horizontal(100.0, 300.0, true, 2.5), start);
        create_bat_with_curve(&mut world, Curve::horizontal(150.0, 350.0, true, 2.5), start);
    }

    for start in float_iter(3.0, 10.0, 0.25) {
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
}

fn create_bat_with_curve(mut world: &mut World, curve: components::Curve, start: f32) {
    entity_with_curve(&mut world, curve)
        .with(components::Image::from(graphics::Image::Bat))
        .with(components::FrozenUntil(start))
        .with(components::DieOffscreen)
        .build();
}

fn entity_with_curve(mut world: &mut World, curve: components::Curve) -> EntityBuilder {
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
