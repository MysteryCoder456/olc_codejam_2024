use bevy::{
    color::palettes::css::*,
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{Line, Rectangle, RectangleOrigin, RegularPolygon, RegularPolygonFeature},
};

use crate::{process::Process, MousePosition};

#[derive(Resource)]
struct TrackPlacementConfig {
    from: Option<Vec2>,
    mouse_distance_threshold: f32,
}

#[derive(Component)]
struct Track {
    from: Vec2,
    to: Vec2,
}

#[derive(Component)]
struct TrackEnd;

#[derive(Component)]
struct TrackPlacementIndicator;

#[derive(Component)]
struct MemoryBus {
    commute_timer: Timer,
    current_commute: Option<(Vec2, Vec2)>, // (from, to)
}

#[derive(Component)]
struct MemoryBusStation;

pub struct MemoryBusPlugin;
impl Plugin for MemoryBusPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TrackPlacementConfig {
            from: None,
            mouse_distance_threshold: 24.0,
        })
        .add_systems(
            Startup,
            (
                spawn_memory_bus.after(spawn_memory_bus_station),
                spawn_memory_bus_station,
                spawn_track_end.after(spawn_memory_bus_station),
                spawn_track_placement_indicator,
            ),
        )
        .add_systems(
            Update,
            (
                place_track,
                track_placement_indicator_position.after(place_track),
            ),
        )
        .add_systems(FixedUpdate, memory_bus_commute);
    }
}

fn spawn_memory_bus(q_station: Query<&Transform, With<MemoryBusStation>>, mut commands: Commands) {
    let station_pos = q_station.single().translation;

    commands.spawn((
        MemoryBus {
            commute_timer: Timer::from_seconds(5.0, TimerMode::Repeating),
            current_commute: None,
        },
        ShapeBundle {
            path: GeometryBuilder::build_as(&Rectangle {
                extents: Vec2::new(16.0, 16.0),
                origin: RectangleOrigin::Center,
            }),
            spatial: SpatialBundle {
                transform: Transform::from_translation(station_pos.with_z(30.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        Fill::color(GREEN),
    ));
}

fn memory_bus_commute(
    time: Res<Time>,
    q_tracks: Query<&Track>,
    mut q_memory_bus: Query<(&mut Transform, &mut MemoryBus)>,
) {
    let (mut bus_tf, mut bus) = q_memory_bus.single_mut();

    bus.commute_timer.tick(time.delta());

    // Find and "attach" to next track piece
    if bus.commute_timer.just_finished() {
        let next_track = q_tracks
            .iter()
            .filter_map(|track| {
                if track.from.distance(bus_tf.translation.truncate()) <= 4.0 {
                    Some(track)
                } else {
                    None
                }
            })
            .next();

        if let Some(next_track) = next_track {
            let from = next_track.from;
            let to = next_track.to;

            bus.current_commute = Some((from, to));
        } else {
            // TODO: Bus probably reached end of track, so retrace steps and go backwards
            bus.current_commute = None;
        }
    }

    // Lerp from `from` to `to`
    if let Some((from, to)) = bus.current_commute {
        let elapsed = bus.commute_timer.elapsed_secs();
        let duration = bus.commute_timer.duration().as_secs_f32();
        let pos = from + (to - from) * elapsed / duration;

        bus_tf.translation.x = pos.x;
        bus_tf.translation.y = pos.y;
    }
}

fn spawn_memory_bus_station(mut commands: Commands) {
    commands.spawn((
        MemoryBusStation,
        ShapeBundle {
            path: GeometryBuilder::build_as(&RegularPolygon {
                sides: 4,
                center: Vec2::ZERO,
                feature: RegularPolygonFeature::Apothem(16.0),
            }),
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 20.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        Fill::color(DARK_GREEN.with_alpha(0.3)),
        Stroke::new(GREEN, 2.0),
    ));
}

fn spawn_track_end(q_station: Query<&Transform, With<MemoryBusStation>>, mut commands: Commands) {
    let station_pos = q_station.single().translation;

    commands.spawn((
        TrackEnd,
        TransformBundle::from_transform(Transform::from_translation(station_pos)),
    ));
}

fn spawn_track_placement_indicator(mut commands: Commands) {
    commands.spawn((
        TrackPlacementIndicator,
        ShapeBundle {
            path: GeometryBuilder::build_as(&Line(Vec2::ZERO, Vec2::ZERO)),
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            ..Default::default()
        },
        Stroke::new(WHITE.with_alpha(0.3), 4.0),
    ));
}

fn track_placement_indicator_position(
    mouse_pos: Res<MousePosition>,
    placement_config: Res<TrackPlacementConfig>,
    mut q_placement_indicator: Query<(&Visibility, &mut Path), With<TrackPlacementIndicator>>,
) {
    let (indicator_visibility, mut indicator_path) = q_placement_indicator.single_mut();
    if indicator_visibility == Visibility::Hidden || !mouse_pos.is_changed() {
        return;
    }

    if let Some(from) = placement_config.from {
        *indicator_path = GeometryBuilder::build_as(&Line(from, mouse_pos.0));
    }
}

// TODO: Prevent overlapping tracks
fn place_track(
    mouse_pos: Res<MousePosition>,
    q_process: Query<&Transform, With<Process>>,
    mut q_track_end: Query<&mut Transform, (With<TrackEnd>, Without<Process>)>,
    mut q_placement_indicator: Query<&mut Visibility, With<TrackPlacementIndicator>>,
    mut placement_config: ResMut<TrackPlacementConfig>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut commands: Commands,
) {
    let mut track_end_tf = q_track_end.single_mut();
    let mut indicator_visibility = q_placement_indicator.single_mut();

    for _ in mouse_button_events
        .read()
        .filter(|ev| ev.button == MouseButton::Left && ev.state == ButtonState::Pressed)
    {
        if let Some(from) = placement_config.from {
            placement_config.from = None;
            *indicator_visibility = Visibility::Hidden;

            // Get the position of any process entity near the mouse, otherwise return mouse
            // position
            let to = q_process
                .iter()
                .filter_map(|tf| {
                    let pos = tf.translation.truncate();
                    if mouse_pos.0.distance(pos) <= placement_config.mouse_distance_threshold {
                        Some(pos)
                    } else {
                        None
                    }
                })
                .next()
                .or(Some(mouse_pos.0))
                .unwrap();

            // Update track end
            track_end_tf.translation.x = to.x;
            track_end_tf.translation.y = to.y;

            // Place down track
            commands.spawn((
                Track { from, to },
                ShapeBundle {
                    path: GeometryBuilder::build_as(&Line(from, to)),
                    spatial: SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Stroke::new(WHITE, 4.0),
            ));
        } else {
            let track_end_pos = track_end_tf.translation.truncate();
            let max_dist = placement_config.mouse_distance_threshold;

            if mouse_pos.0.distance(track_end_pos) <= max_dist {
                placement_config.from = Some(track_end_pos);
                *indicator_visibility = Visibility::Inherited;
            }
        }
    }
}
