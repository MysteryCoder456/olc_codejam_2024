use std::time::Duration;

use crate::{track::Track, BusStop};
use bevy::{color::palettes::css::*, prelude::*};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{Rectangle, RegularPolygon, RegularPolygonFeature},
};

#[derive(Event)]
pub struct SpawnBusEvent;

#[derive(Event)]
pub struct SpawnBusStationEvent {
    pub position: Vec2,
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
            .add_systems(FixedUpdate, bus_commutes);
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
struct BusStation;

fn spawn_bus(
    mut commands: Commands,
    mut events: EventReader<SpawnBusEvent>,
    station_query: Single<(Entity, &Transform), With<BusStation>>,
) {
    let (station_entity, station_tf) = *station_query;
    let shape = Rectangle {
        extents: Vec2::new(24.0, 16.0),
        origin: RectangleOrigin::Center,
        radii: Some(BorderRadii::single(2.0)),
    };

    for _event in events.read() {
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                transform: Transform::from_translation(station_tf.translation.with_z(20.0)),
                ..Default::default()
            },
            Fill::color(ORANGE),
            Bus {
                commute_timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating),
                stop_wait_timer: Timer::new(Duration::from_secs(8), TimerMode::Repeating),
                commute_state: CommuteState::Waiting(station_entity),
            },
        ));
    }
}

fn spawn_bus_station(mut commands: Commands, mut events: EventReader<SpawnBusStationEvent>) {
    let shape = RegularPolygon {
        sides: 6,
        center: Vec2::ZERO,
        feature: RegularPolygonFeature::Radius(24.0),
    };

    for event in events.read() {
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                transform: Transform::from_translation(event.position.extend(10.0)),
                ..Default::default()
            },
            Fill::color(ORANGE_RED),
            Stroke::new(ORANGE_RED.darker(0.1), 4.0),
            BusStation,
            BusStop,
        ));
    }
}

fn bus_commutes(
    time: Res<Time<Fixed>>,
    mut bus_query: Query<(&mut Bus, &mut Transform)>,
    track_query: Query<(Entity, &Track)>,
    stop_query: Query<&Transform, (With<BusStop>, Without<Bus>)>,
) {
    for (mut bus, mut bus_tf) in bus_query.iter_mut() {
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

                if bus.stop_wait_timer.just_finished() {
                    let Ok(stop_tf) = stop_query.get(stop_entity) else {
                        warn!("Bus was waiting at a non-existent stop. This should not happen.");
                        continue;
                    };

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

                    debug!("Bus is starting new commute.");
                    if let Some(track_entity) = track_entity {
                        bus.commute_state = CommuteState::Commuting(track_entity);
                    } else {
                        // TODO: Reached end of track. Reverse commute.
                    }
                    continue;
                }

                // TODO: wait at station and do whatever
            }
        }
    }
}
