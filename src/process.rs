use std::time::Duration;

use crate::{BusStop, Collider, Velocity};
use bevy::{
    color::palettes::css::{DARK_GREEN, GREEN, INDIAN_RED, RED},
    math::bounding::Aabb2d,
    prelude::*,
};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{Rectangle, RectangleOrigin, RegularPolygon, RegularPolygonFeature},
};
use rand::prelude::*;

#[derive(Event)]
pub struct SpawnProcessEvent {
    pub position: Vec2,
}

#[derive(Event)]
pub struct DespawnGarbageIndicatorAtProcessEvent {
    pub process_entity: Entity,
}

#[derive(Component)]
pub struct ProcessMemory {
    pub memory: f32,
}

#[derive(Component)]
pub struct GarbageIndicator;

pub struct ProcessPlugin;

impl Plugin for ProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnProcessEvent>()
            .add_event::<SpawnGarbageIndicatorEvent>()
            .add_event::<DespawnGarbageIndicatorAtProcessEvent>()
            .add_systems(
                Update,
                (
                    spawn_processes.run_if(on_event::<SpawnProcessEvent>),
                    spawn_garbage_indicators.run_if(on_event::<SpawnGarbageIndicatorEvent>),
                    despawn_garbage_indicators
                        .run_if(on_event::<DespawnGarbageIndicatorAtProcessEvent>),
                    process_memory_indicator,
                ),
            )
            .add_systems(FixedUpdate, (process_memory_usage, process_out_of_memory));
    }
}

#[derive(Event)]
struct SpawnGarbageIndicatorEvent {
    process_entity: Entity,
}

enum MemoryState {
    Idle,
    InUse { usage_per_second: f32 },
}

#[derive(Component)]
struct Process {
    memory_state: MemoryState,
    memory_usage_timer: Timer,
    memory_idle_timer: Timer,
    garbage_spawn_timer: Timer,
    out_of_memory_timer: Timer,
}

fn spawn_processes(mut commands: Commands, mut events: EventReader<SpawnProcessEvent>) {
    let mut rng = rand::thread_rng();
    let shape = RegularPolygon {
        sides: 6,
        center: Vec2::ZERO,
        feature: RegularPolygonFeature::Radius(24.0),
    };

    for event in events.read() {
        debug!("Spawning process at {:?}", event.position);
        commands
            .spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shape),
                    transform: Transform::from_translation(event.position.extend(10.0)),
                    ..Default::default()
                },
                Fill::color(GREEN),
                Stroke::new(DARK_GREEN, 4.0),
                Process {
                    memory_state: MemoryState::Idle,
                    memory_usage_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                    memory_idle_timer: Timer::from_seconds(
                        rng.gen_range(8.5..=11.5),
                        TimerMode::Repeating,
                    ),
                    garbage_spawn_timer: Timer::from_seconds(1.8, TimerMode::Repeating),
                    out_of_memory_timer: Timer::from_seconds(30.0, TimerMode::Once),
                },
                ProcessMemory { memory: 50.0 },
                BusStop,
            ))
            .with_child((
                Text2d::new("0"),
                TextColor(Color::WHITE),
                Transform {
                    translation: Vec3::new(0.0, 0.0, 2.0),
                    ..Default::default()
                },
            ));
    }
}

fn spawn_garbage_indicators(
    mut commands: Commands,
    mut events: EventReader<SpawnGarbageIndicatorEvent>,
) {
    let shape = Rectangle {
        extents: Vec2::new(16.0, 16.0),
        origin: RectangleOrigin::Center,
        radii: Some(BorderRadii::single(2.0)),
    };

    let mut rng = thread_rng();
    let friction = 1.75;

    for event in events.read() {
        let initial_speed = rng.gen_range(70.0..=100.0);
        let velocity =
            initial_speed * Vec2::from_angle(rng.gen_range(0.0..(2.0 * std::f32::consts::PI)));

        // Prevent z-fighting among garbage indicators
        let relative_z = rng.gen_range(0.0..1.0);

        commands.entity(event.process_entity).with_child((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0 + relative_z)),
                ..Default::default()
            },
            Fill::color(INDIAN_RED),
            Stroke::new(INDIAN_RED.darker(0.1), 1.5),
            GarbageIndicator,
            Velocity {
                velocity,
                friction: Some(friction),
            },
            Collider(Aabb2d::new(Vec2::ZERO, shape.extents / 2.0)),
        ));
    }
}

fn despawn_garbage_indicators(
    mut commands: Commands,
    mut events: EventReader<DespawnGarbageIndicatorAtProcessEvent>,
    process_query: Query<&Children, With<Process>>,
    garbage_query: Query<Entity, With<GarbageIndicator>>,
) {
    for event in events.read() {
        let Ok(process_children) = process_query.get(event.process_entity) else {
            continue;
        };

        // Despawn the first child with the GarbageIndicator component
        for child in process_children.iter() {
            if garbage_query.contains(*child) {
                commands.entity(*child).despawn_recursive();
                break;
            }
        }
    }
}

fn process_memory_indicator(
    process_query: Query<(&Process, &ProcessMemory, &Children), Changed<ProcessMemory>>,
    mut text_query: Query<(&mut Text2d, &mut TextColor)>,
) {
    for (process, process_memory, children) in process_query.iter() {
        if let Some((mut memory_indicator_text, mut memory_indicator_color)) =
            children.first().and_then(|e| text_query.get_mut(*e).ok())
        {
            memory_indicator_text.0 = if process_memory.memory > 0.0 {
                process_memory.memory.ceil().to_string()
            } else {
                "0".to_owned()
            };
            memory_indicator_color.0 = match process.memory_state {
                MemoryState::Idle => Color::WHITE,
                MemoryState::InUse { .. } => Color::Srgba(RED),
            };
        } else {
            warn!(
                "Process does not seem to have a memory indicator child entity with a Text2d component!"
            );
        }
    }
}

fn process_memory_usage(
    time: Res<Time<Fixed>>,
    mut garbage_indicator_events: EventWriter<SpawnGarbageIndicatorEvent>,
    mut process_query: Query<(Entity, &mut Process, &mut ProcessMemory)>,
) {
    let mut rng = rand::thread_rng();

    for (process_entity, mut process, mut process_memory) in process_query.iter_mut() {
        match process.memory_state {
            MemoryState::Idle => {
                process.memory_idle_timer.tick(time.delta());

                if process.memory_idle_timer.just_finished() && process_memory.memory > 0.0 {
                    // Memory is now "in use"
                    let total_usage = rng
                        .gen_range::<f32, _>(10.0..=15.0)
                        .min(process_memory.memory);
                    let usage_per_second =
                        total_usage / process.memory_usage_timer.duration().as_secs_f32();
                    process.memory_state = MemoryState::InUse { usage_per_second };
                }
            }
            MemoryState::InUse { usage_per_second } => {
                process.memory_usage_timer.tick(time.delta());
                process.garbage_spawn_timer.tick(time.delta());

                // "Consume" memory
                let usage = usage_per_second * time.delta_secs();
                process_memory.memory -= usage;

                // "Produce" garbage memory
                if process.garbage_spawn_timer.just_finished() {
                    garbage_indicator_events.send(SpawnGarbageIndicatorEvent { process_entity });
                }

                if process.memory_usage_timer.just_finished() {
                    // Stay idle for a random duration
                    let new_duration = Duration::from_secs_f32(rng.gen_range(5.0..=10.0));
                    process.memory_idle_timer.set_duration(new_duration);
                    process.memory_state = MemoryState::Idle;
                }
            }
        }
    }
}

fn process_out_of_memory(
    time: Res<Time<Fixed>>,
    mut gizmos: Gizmos,
    mut process_query: Query<(&mut Process, &ProcessMemory, &Transform)>,
) {
    for (mut process, process_memory, process_tf) in process_query.iter_mut() {
        if process_memory.memory > 0.0 {
            process.out_of_memory_timer.reset();
            continue;
        }
        process.out_of_memory_timer.tick(time.delta());

        // Draw timer indicator
        let elapsed_time = process.out_of_memory_timer.elapsed_secs();
        let total_time = process.out_of_memory_timer.duration().as_secs_f32();
        let times =
            (0..=(360.0 * elapsed_time / total_time) as i32).map(|n| (n as f32).to_radians());
        let curve = FunctionCurve::new(Interval::EVERYWHERE, |t| {
            process_tf.translation.truncate() + Vec2::from(t.sin_cos()) * 32.0
        });
        gizmos.curve_2d(curve, times, DARK_GREEN);

        // Times up!
        if process.out_of_memory_timer.just_finished() {
            debug!("Process out of memory");
            // TODO: game over
        }
    }
}
