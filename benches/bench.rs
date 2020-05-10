use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hectic_rs::{systems, resources::*};
use specs::prelude::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut world = World::new();

    hectic_rs::register_components(&mut world);

    //world.insert(ControlsState::load());
    //world.insert(buffer_renderer);
    world.insert(GameTime::default());
    world.insert(PlayerPositions::default());
    world.insert(Mode::default());

    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::FinishStage, "FinishStage", &[])
        .with(systems::MoveBosses, "MoveBosses", &[])
        .with(systems::ExplosionImages, "ExplosionImages", &[])
        //.with(systems::TogglePaused, "TogglePaused", &[])
        .with(systems::KillOffscreen, "KillOffscreen", &[])
        .with(systems::ExpandBombs, "ExpandCircles", &[])
        .with(systems::MoveEntities, "MoveEntities", &[])
        .with(systems::CollectOrbs, "CollectOrbs", &[])
        //.with(systems::Control, "Control", &[])
        .with(systems::SetPlayerPositions, "SetPlayerPositions", &[])
        .with(systems::FireBullets, "FireBullets", &[])
        .with(systems::RepeatBackgroundLayers, "RepeatBackgroundLayers", &[])
        .with(systems::TickTime, "TickTime", &[])
        .with(systems::StartTowardsPlayer, "StartTowardsPlayer", &["TickTime"])
        .with(systems::AddOnscreen, "AddOnscreen", &[])
        .with(systems::Collisions, "Collisions", &[])
        .build();

    c.bench_function("fib 20", |b| {
        {
            let (entities, updater, mut time): (Entities, Read<LazyUpdate>, Write<GameTime>) = world.system_data();
            hectic_rs::stages::stage_one(&entities, &updater, true, &mut time.total_time);
        }

        b.iter(|| {
            for _ in 0 .. 60 * 15 {
                dispatcher.dispatch(&world);
                world.maintain();
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
