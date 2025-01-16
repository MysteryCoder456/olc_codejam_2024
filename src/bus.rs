use std::time::Duration;

use crate::{
    process::{DespawnGarbageIndicatorAtProcessEvent, GarbageIndicator, ProcessMemory},
    track::Track,
    BusStop, Collider,
};
use bevy::{
    color::palettes::{css::INDIAN_RED, tailwind::CYAN_600},
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{Rectangle, RegularPolygon, RegularPolygonFeature},
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum StationType {
    Memory,
    GarbageCollector,
}

#[derive(Event)]
pub struct SpawnBusEvent {
    pub station_type: StationType,
}

#[derive(Event)]
pub struct SpawnBusStationEvent {
    pub position: Vec2,
    pub station_type: StationType,
}

pub struct BusPlugin;

impl Plugin for BusPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnBusEvent>()
            .add_event::<SpawnBusStationEvent>()
            .add_systems(
                Update,
                (
                    spawn_bus_station.run_if(on_event::<SpawnBusStationEvent>),
                    spawn_bus
                        .run_if(on_event::<SpawnBusEvent>)
                        .after(spawn_bus_station),
                ),
            )
            .add_systems(FixedUpdate, (bus_commutes, bus_garbage_collisions));
    }
}

enum CommuteState {
    Commuting(Entity),
    Waiting(Entity),
}

#[derive(Component)]
struct Bus {
    commute_timer: Timer,
    stop_wait_timer: Timer,
    commute_state: CommuteState,
}

#[derive(Component)]
enum BusType {
    Memory,
    GarbageCollector(Timer),
}

#[derive(Component)]
struct BusStation {
    station_type: StationType,
}

fn spawn_bus(
    mut commands: Commands,
    mut events: EventReader<SpawnBusEvent>,
    station_query: Query<(Entity, &BusStation, &Transform, &Fill)>,
) {
    let shape = Rectangle {
        extents: Vec2::new(24.0, 16.0),
        origin: RectangleOrigin::Center,
        radii: Some(BorderRadii::single(2.0)),
    };

    for event in events.read() {
        for (station_entity, station, station_tf, station_fill) in station_query.iter() {
            if station.station_type != event.station_type {
                continue;
            }

            let bus_type = match event.station_type {
                StationType::Memory => BusType::Memory,
                StationType::GarbageCollector => {
                    BusType::GarbageCollector(Timer::from_seconds(1.2, TimerMode::Repeating))
                }
            };

            commands.spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shape),
                    transform: Transform::from_translation(station_tf.translation.with_z(20.0)),
                    ..Default::default()
                },
                Fill::color(station_fill.color.with_luminance(0.8)),
                Bus {
                    commute_timer: Timer::new(Duration::from_secs_f32(4.0), TimerMode::Repeating),
                    stop_wait_timer: Timer::new(Duration::from_secs_f32(4.0), TimerMode::Repeating),
                    commute_state: CommuteState::Waiting(station_entity),
                },
                bus_type,
                Collider(Aabb2d::new(Vec2::ZERO, shape.extents / 2.0)),
            ));
            break;
        }
    }
}

fn spawn_bus_station(mut commands: Commands, mut events: EventReader<SpawnBusStationEvent>) {
    let shape = RegularPolygon {
        sides: 6,
        center: Vec2::ZERO,
        feature: RegularPolygonFeature::Radius(24.0),
    };

    for event in events.read() {
        let fill_color = match event.station_type {
            StationType::Memory => CYAN_600,
            StationType::GarbageCollector => INDIAN_RED,
        };

        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                transform: Transform::from_translation(event.position.extend(10.0)),
                ..Default::default()
            },
            Fill::color(fill_color),
            Stroke::new(fill_color.darker(0.1), 4.0),
            BusStation {
                station_type: event.station_type,
            },
            BusStop,
        ));
    }
}

fn bus_commutes(
    time: Res<Time<Fixed>>,
    mut garbage_despawn_events: EventWriter<DespawnGarbageIndicatorAtProcessEvent>,
    track_query: Query<(Entity, &Track)>,
    mut bus_query: Query<(&mut Bus, &mut BusType, &mut Transform)>,
    mut stop_query: Query<(&Transform, Option<&mut ProcessMemory>), (With<BusStop>, Without<Bus>)>,
) {
    for (mut bus, mut bus_type, mut bus_tf) in bus_query.iter_mut() {
        match bus.commute_state {
            CommuteState::Commuting(track_entity) => {
                let Ok((_, track)) = track_query.get(track_entity) else {
                    warn!("Bus is commuting on a non-existent track. This should not happen.");
                    continue;
                };

                bus.commute_timer.tick(time.delta());
                if bus.commute_timer.just_finished() {
                    debug!("Bus is now waiting at stop {:?}", track.destination);
                    bus.commute_state = CommuteState::Waiting(track.destination);
                    bus_tf.translation = track.path.last().unwrap().extend(bus_tf.translation.z);
                    continue;
                }

                // Calculate bus's position on the track based on the elapsed time
                let total_progress =
                    bus.commute_timer.elapsed_secs() / bus.commute_timer.duration().as_secs_f32();
                let line_progress = total_progress * (track.path.len() - 1) as f32 % 1.0;

                let path_idx = (total_progress * (track.path.len() - 1) as f32).floor() as usize;
                let from = track.path[path_idx];
                let to = track.path[path_idx + 1];

                bus_tf.translation = from.lerp(to, line_progress).extend(bus_tf.translation.z);
                bus_tf.rotation = Quat::from_rotation_z((to - from).to_angle());
            }
            CommuteState::Waiting(stop_entity) => {
                bus.stop_wait_timer.tick(time.delta());

                let Ok((stop_tf, process_memory)) = stop_query.get_mut(stop_entity) else {
                    warn!("Bus was waiting at a non-existent stop. This should not happen.");
                    continue;
                };

                // If waiting at a process station, do the necessary actions
                if let Some(mut process_memory) = process_memory {
                    match *bus_type {
                        BusType::Memory => {
                            // Give memory to the process
                            let memory_given = 10.0;
                            process_memory.memory += memory_given * time.delta_secs();
                        }
                        BusType::GarbageCollector(ref mut timer) => {
                            // Collect garbage one at a time
                            timer.tick(time.delta());
                            if timer.just_finished() {
                                garbage_despawn_events.send(
                                    DespawnGarbageIndicatorAtProcessEvent {
                                        process_entity: stop_entity,
                                    },
                                );
                            }
                        }
                    }
                }

                if bus.stop_wait_timer.just_finished() {
                    // Find a track to commute on
                    let track_entity = track_query
                        .iter()
                        .filter_map(|(track_entity, track)| {
                            if track.path[0] == stop_tf.translation.truncate() {
                                Some(track_entity)
                            } else {
                                None
                            }
                        })
                        .next();

                    if let Some(track_entity) = track_entity {
                        debug!("Bus is starting new commute");
                        bus.commute_state = CommuteState::Commuting(track_entity);
                    } else {
                        // TODO: Reached end of track. Reverse commute.
                    }
                }
            }
        }
    }
}

fn bus_garbage_collisions(
    mut commands: Commands,
    garbage_query: Query<(Entity, &Collider<Aabb2d>, &GlobalTransform), With<GarbageIndicator>>,
    bus_query: Query<
        (&BusType, &Collider<Aabb2d>, &GlobalTransform),
        (With<Bus>, Without<GarbageIndicator>, Changed<Transform>),
    >,
) {
    for (bus_type, bus_collider, bus_tf) in bus_query.iter() {
        // FIXME: panics when bus rotates
        //let bus_volume = bus_collider.0.transformed_by(
        //    bus_tf.translation().truncate(),
        //    bus_tf.rotation().to_axis_angle().1,
        //);

        // HACK: Manually transform the volume (workaround for the FIXME above)
        let rotation = bus_tf.rotation().to_axis_angle().1;
        let rot_mat = Mat2::from_cols(
            Vec2::new(rotation.cos(), rotation.sin()),
            Vec2::new(-rotation.sin(), rotation.cos()),
        );
        let half_size = rot_mat * bus_collider.0.half_size();
        let bus_volume = Aabb2d::new(bus_tf.translation().truncate(), half_size.abs());

        for (garbage_entity, garbage_collider, garbage_tf) in garbage_query.iter() {
            let garbage_volume = garbage_collider
                .0
                .translated_by(garbage_tf.translation().truncate());

            if garbage_volume.intersects(&bus_volume) {
                debug!(
                    "Bus collided with garbage at {:?}",
                    garbage_tf.translation().truncate()
                );
                commands.entity(garbage_entity).despawn_recursive();

                match *bus_type {
                    BusType::Memory => {
                        // TODO: Bus has crashed
                    }
                    BusType::GarbageCollector(_) => {
                        // Collect the garbage (already achieve by despawn)
                    }
                }
            }
        }
    }
}
