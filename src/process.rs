use std::time::Duration;

use crate::BusStop;
use bevy::{
    color::palettes::css::{DARK_GREEN, GREEN},
    prelude::*,
};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{RegularPolygon, RegularPolygonFeature},
};
use rand::prelude::*;

#[derive(Event)]
pub struct SpawnProcessEvent {
    pub position: Vec2,
}

pub struct ProcessPlugin;

impl Plugin for ProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnProcessEvent>()
            .add_systems(
                Update,
                (
                    spawn_processes.run_if(on_event::<SpawnProcessEvent>),
                    process_memory_indicator,
                ),
            )
            .add_systems(FixedUpdate, (process_memory_usage, process_out_of_memory));
    }
}

#[derive(Component)]
struct Process {
    memory: f32,
    memory_usage_timer: Timer,
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
                    memory: 50.0,
                    memory_usage_timer: Timer::from_seconds(
                        rng.gen_range(15.0..=30.0),
                        TimerMode::Repeating,
                    ),
                    out_of_memory_timer: Timer::from_seconds(60.0, TimerMode::Once),
                },
                BusStop,
            ))
            .with_child((
                Text2d::new("0"),
                Transform {
                    translation: Vec3::new(0.0, 0.0, 1.0),
                    ..Default::default()
                },
            ));
    }
}

fn process_memory_indicator(
    process_query: Query<(&Process, &Children), Changed<Process>>,
    mut text_query: Query<&mut Text2d>,
) {
    for (process, children) in process_query.iter() {
        if let Some(mut memory_indicator_text) =
            children.first().and_then(|e| text_query.get_mut(*e).ok())
        {
            memory_indicator_text.0 = process.memory.ceil().to_string();
        } else {
            warn!(
                "Process does not seem to have a memory indicator child entity with a Text2d component!"
            );
        }
    }
}

fn process_memory_usage(time: Res<Time<Fixed>>, mut process_query: Query<&mut Process>) {
    let mut rng = rand::thread_rng();

    for mut process in process_query.iter_mut() {
        process.memory_usage_timer.tick(time.delta());

        if process.memory_usage_timer.just_finished() && process.memory > 0.0 {
            // "Use" memory
            let usage = rng.gen_range::<f32, _>(5.0..=20.0).min(process.memory);
            process.memory -= usage;
            // TODO: increase garbage memory counter
            debug!(
                "Process used {} memory, {} remaining",
                usage, process.memory
            );

            // Update timer with a random duration
            let new_duration = Duration::from_secs_f32(rng.gen_range(10.0..=20.0));
            process.memory_usage_timer.set_duration(new_duration);
        }
    }
}

fn process_out_of_memory(
    time: Res<Time<Fixed>>,
    mut gizmos: Gizmos,
    mut process_query: Query<(&mut Process, &Transform)>,
) {
    for (mut process, process_tf) in process_query.iter_mut() {
        if process.memory > 0.0 {
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
