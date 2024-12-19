use crate::BusStop;
use bevy::{
    color::palettes::css::{DARK_GREEN, GREEN},
    prelude::*,
};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{RegularPolygon, RegularPolygonFeature},
};

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
                spawn_processes.run_if(on_event::<SpawnProcessEvent>),
            )
            .add_systems(FixedUpdate, process_out_of_memory);
    }
}

#[derive(Component)]
struct Process {
    memory: f32,
    out_of_memory_timer: Timer,
}

fn spawn_processes(mut commands: Commands, mut events: EventReader<SpawnProcessEvent>) {
    let shape = RegularPolygon {
        sides: 6,
        center: Vec2::ZERO,
        feature: RegularPolygonFeature::Radius(24.0),
    };

    for event in events.read() {
        debug!("Spawning process at {:?}", event.position);
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                transform: Transform::from_translation(event.position.extend(10.0)),
                ..Default::default()
            },
            Fill::color(GREEN),
            Stroke::new(DARK_GREEN, 4.0),
            Process {
                memory: 100.0,
                out_of_memory_timer: Timer::from_seconds(60.0, TimerMode::Once),
            },
            BusStop,
        ));
    }
}

fn process_memory_usage() {
    // TODO: use available memory and produce garbage memory
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
