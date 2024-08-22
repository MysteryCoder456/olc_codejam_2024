use bevy::{color::palettes::css::*, prelude::*};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{RegularPolygon, RegularPolygonFeature},
};
use rand::prelude::*;

const PROCESS_SHAPE: RegularPolygon = RegularPolygon {
    sides: 6,
    feature: RegularPolygonFeature::Apothem(16.0),
    center: Vec2::ZERO,
};

#[derive(Resource)]
struct ProcessSpawnConfig {
    timer: Timer,
}

#[derive(Component)]
pub struct Process;

pub struct ProcessPlugin;
impl Plugin for ProcessPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProcessSpawnConfig {
            timer: Timer::from_seconds(30.0, TimerMode::Repeating),
        })
        .add_systems(Startup, spawn_initial_processes);
        //.add_systems(FixedUpdate, spawn_process);
    }
}

fn spawn_initial_processes(mut commands: Commands) {
    let positions = [
        Vec2::new(-64.0, -64.0),
        Vec2::new(0.0, 128.0),
        Vec2::new(64.0, 0.0),
    ];

    for pos in positions {
        commands.spawn((
            Process,
            ShapeBundle {
                path: GeometryBuilder::build_as(&PROCESS_SHAPE),
                spatial: SpatialBundle::from_transform(Transform {
                    translation: pos.extend(20.0),
                    ..Default::default()
                }),
                ..Default::default()
            },
            Fill::color(DARK_GREEN.with_alpha(0.3)),
            Stroke::new(GREEN, 2.0),
        ));
    }
}

fn spawn_process(
    time: Res<Time>,
    mut spawn_config: ResMut<ProcessSpawnConfig>,
    mut commands: Commands,
) {
    // Process spawn timer
    spawn_config.timer.tick(time.delta());
    if !spawn_config.timer.just_finished() {
        return;
    }

    // Generate random position
    let mut rng = rand::thread_rng();
    let random_pos = Vec2::new(rng.gen_range(-512.0..=512.0), rng.gen_range(-384.0..=384.0));

    commands.spawn((
        Process,
        ShapeBundle {
            path: GeometryBuilder::build_as(&PROCESS_SHAPE),
            spatial: SpatialBundle::from_transform(Transform {
                translation: random_pos.extend(0.0),
                ..Default::default()
            }),
            ..Default::default()
        },
        Fill::color(DARK_GREEN.with_alpha(0.3)),
        Stroke::new(GREEN, 2.0),
    ));
}
