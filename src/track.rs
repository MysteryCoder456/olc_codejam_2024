use crate::{process::Process, CursorPosition};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_prototype_lyon::prelude::*;

pub struct TrackPlugin;

impl Plugin for TrackPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TrackPlacementConfig {
            initial_position: None,
            source: None,
            process_distance_threshold: 32.0,
            thickness: 6.0,
            color: Color::srgb_u8(184, 115, 51),
        })
        .add_systems(
            Update,
            (
                draw_placement_indicator.run_if(resource_changed::<CursorPosition>),
                place_track.run_if(input_just_pressed(MouseButton::Left)),
            ),
        );
    }
}

#[derive(Resource)]
struct TrackPlacementConfig {
    initial_position: Option<Vec2>,
    source: Option<Entity>,
    process_distance_threshold: f32,
    thickness: f32,
    color: Color,
}

#[derive(Component)]
struct Track {
    path: Vec<Vec2>,
    source: Entity,
    destination: Entity,
}

fn draw_placement_indicator(
    mut gizmos: Gizmos,
    cursor_position: Res<CursorPosition>,
    placement_config: Res<TrackPlacementConfig>,
    process_query: Query<&Transform, With<Process>>,
) {
    let Some(initial_position) = placement_config.initial_position else {
        return;
    };

    // Check if the cursor is on or near a process entity
    let process_position = process_query
        .iter()
        .map(|transform| transform.translation.truncate())
        .min_by_key(|position| {
            let diff = cursor_position.0 - *position;
            (diff.x * diff.x + diff.y * diff.y) as i32
        })
        .and_then(|p| {
            if p.distance(cursor_position.0) < placement_config.process_distance_threshold {
                Some(p)
            } else {
                None
            }
        });

    let to = if let Some(process_position) = process_position {
        process_position
    } else {
        cursor_position.0
    };

    let color = placement_config.color.with_alpha(0.5);
    let diff = to - initial_position;

    let corner = if diff.x.abs() >= diff.y.abs() {
        Vec2::new(to.x, initial_position.y)
    } else {
        Vec2::new(initial_position.x, to.y)
    };

    gizmos.line_2d(initial_position, corner, color);
    gizmos.line_2d(corner, to, color);
}

fn place_track(
    mut commands: Commands,
    mut placement_config: ResMut<TrackPlacementConfig>,
    cursor_position: Res<CursorPosition>,
    process_query: Query<(Entity, &Transform), With<Process>>,
) {
    // Check if the cursor is on or near a process entity
    let Some((process, process_position)) = process_query
        .iter()
        .map(|(e, tf)| (e, tf.translation.truncate()))
        .min_by_key(|(_e, pos)| {
            let diff = cursor_position.0 - *pos;
            (diff.x * diff.x + diff.y * diff.y) as i32
        })
        .and_then(|(e, pos)| {
            if pos.distance(cursor_position.0) < placement_config.process_distance_threshold {
                Some((e, pos))
            } else {
                None
            }
        })
    else {
        return;
    };

    let (Some(from), Some(source)) = (placement_config.initial_position, placement_config.source)
    else {
        debug!("Setting initial position to {:?}", cursor_position.0);
        placement_config.initial_position = Some(process_position);
        placement_config.source = Some(process);
        return;
    };

    let destination = process;
    let to = process_position;

    // Prevent self-connections
    if source == destination || from == to {
        return;
    }

    debug!("Placed track from {:?} to {:?}", from, to);

    let radius = placement_config.thickness / 2.0;
    let diff = to - from;
    let x_diff_greater = diff.x.abs() >= diff.y.abs();
    let corner = if x_diff_greater {
        Vec2::new(to.x, from.y)
    } else {
        Vec2::new(from.x, to.y)
    };

    let points = if diff.x >= 0.0 && diff.y >= 0.0 {
        vec![
            from + Vec2::new(-radius, radius * if x_diff_greater { 1.0 } else { -1.0 }),
            corner + Vec2::new(-radius, radius),
            to + Vec2::new(radius * if x_diff_greater { -1.0 } else { 1.0 }, radius),
            to + Vec2::new(radius, radius * if x_diff_greater { 1.0 } else { -1.0 }),
            corner + Vec2::new(radius, -radius),
            from + Vec2::new(radius * if x_diff_greater { -1.0 } else { 1.0 }, -radius),
        ]
    } else if diff.x < 0.0 && diff.y >= 0.0 {
        vec![
            from + Vec2::new(-radius, -radius),
            corner + Vec2::new(-radius, -radius),
            to + Vec2::new(-radius, -radius),
            to + Vec2::new(-radius, radius) * if x_diff_greater { -1.0 } else { 1.0 },
            corner + Vec2::new(radius, radius),
            from + Vec2::new(radius, -radius) * if x_diff_greater { -1.0 } else { 1.0 },
        ]
    } else if diff.x >= 0.0 && diff.y < 0.0 {
        vec![
            from + Vec2::new(-radius, radius),
            corner + Vec2::new(radius, radius) * if x_diff_greater { 1.0 } else { -1.0 },
            to + Vec2::new(radius, -radius),
            to + Vec2::new(radius, radius) * if x_diff_greater { -1.0 } else { 1.0 },
            corner + Vec2::new(radius, radius) * if x_diff_greater { -1.0 } else { 1.0 },
            from + Vec2::new(radius, radius) * if x_diff_greater { -1.0 } else { 1.0 },
        ]
    } else {
        vec![
            from + Vec2::new(radius * if x_diff_greater { 1.0 } else { -1.0 }, radius),
            corner + Vec2::new(-radius, radius),
            to + Vec2::new(-radius, radius * if x_diff_greater { -1.0 } else { 1.0 }),
            to + Vec2::new(radius * if x_diff_greater { 1.0 } else { -1.0 }, -radius),
            corner + Vec2::new(radius, -radius),
            from + Vec2::new(radius, radius * if x_diff_greater { -1.0 } else { 1.0 }),
        ]
    };
    let shape = shapes::RoundedPolygon {
        points,
        closed: true,
        radius,
    };

    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        Fill::color(placement_config.color),
        Track {
            path: vec![from, corner, to],
            source,
            destination,
        },
    ));

    placement_config.initial_position = None;
}
