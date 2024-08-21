use bevy::{
    color::palettes::css::*,
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{Line, RegularPolygon, RegularPolygonFeature},
};

use crate::MousePosition;

pub struct MemoryBusPlugin;
impl Plugin for MemoryBusPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TrackPlacementConfig {
            from: None,
            next_from: None,
            next_from_theshold: 8.0,
        })
        .add_systems(
            Startup,
            (spawn_memory_bus_station, spawn_track_placement_indicator),
        )
        .add_systems(
            Update,
            (place_track, track_placement_indicator_position).chain(),
        );
    }
}

#[derive(Resource)]
struct TrackPlacementConfig {
    from: Option<Vec2>,
    next_from: Option<Vec2>,
    next_from_theshold: f32,
}

#[derive(Component)]
struct Track;

#[derive(Component)]
struct TrackPlacementIndicator;

#[derive(Component)]
struct MemoryBusStation;

fn spawn_memory_bus_station(
    mut placement_config: ResMut<TrackPlacementConfig>,
    mut commands: Commands,
) {
    let spawn_location = Vec2::ZERO;
    placement_config.next_from = Some(spawn_location);

    commands.spawn((
        MemoryBusStation,
        ShapeBundle {
            path: GeometryBuilder::build_as(&RegularPolygon {
                sides: 4,
                center: Vec2::ZERO,
                feature: RegularPolygonFeature::Apothem(16.0),
            }),
            spatial: SpatialBundle {
                transform: Transform::from_translation(spawn_location.extend(0.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        Fill::color(DARK_GREEN.with_alpha(0.3)),
        Stroke::new(GREEN, 2.0),
    ));
}

fn spawn_track_placement_indicator(mut commands: Commands) {
    commands.spawn((
        TrackPlacementIndicator,
        ShapeBundle {
            path: GeometryBuilder::build_as(&Line(Vec2::ZERO, Vec2::ZERO)),
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            ..Default::default()
        },
        Stroke::new(WHITE.with_alpha(0.3), 8.0),
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

fn place_track(
    mouse_pos: Res<MousePosition>,
    mut q_placement_indicator: Query<&mut Visibility, With<TrackPlacementIndicator>>,
    mut placement_config: ResMut<TrackPlacementConfig>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut commands: Commands,
) {
    let mut indicator_visibility = q_placement_indicator.single_mut();

    if placement_config.next_from.is_none() {
        return;
    }
    let next_placement = placement_config.next_from.unwrap();

    for _ in mouse_button_events
        .read()
        .filter(|ev| ev.button == MouseButton::Left && ev.state == ButtonState::Pressed)
    {
        if let Some(from) = placement_config.from {
            placement_config.from = None;
            *indicator_visibility = Visibility::Hidden;

            let to = mouse_pos.0;
            placement_config.next_from = Some(to);

            // Place down track
            commands.spawn((
                Track,
                ShapeBundle {
                    path: GeometryBuilder::build_as(&Line(from, to)),
                    spatial: SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Stroke::new(WHITE, 8.0),
            ));
        } else if mouse_pos.0.distance(next_placement) <= placement_config.next_from_theshold {
            placement_config.from = Some(mouse_pos.0);
            *indicator_visibility = Visibility::Inherited;
        }
    }
}
