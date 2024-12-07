use crate::CursorPosition;
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_prototype_lyon::prelude::*;

pub struct TrackPlugin;

impl Plugin for TrackPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TrackPlacementConfig {
            initial_position: None,
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
    thickness: f32,
    color: Color,
}

#[derive(Component)]
struct Track {
    path: Vec<Vec2>,
}

fn draw_placement_indicator(
    cursor_position: Res<CursorPosition>,
    placement_config: Res<TrackPlacementConfig>,
    mut gizmos: Gizmos,
) {
    let Some(initial_position) = placement_config.initial_position else {
        return;
    };

    let color = placement_config.color.with_alpha(0.5);
    let diff = cursor_position.0 - initial_position;

    let corner = if diff.x.abs() >= diff.y.abs() {
        Vec2::new(cursor_position.0.x, initial_position.y)
    } else {
        Vec2::new(initial_position.x, cursor_position.0.y)
    };

    gizmos.line_2d(initial_position, corner, color);
    gizmos.line_2d(corner, cursor_position.0, color);
}

fn place_track(
    cursor_position: Res<CursorPosition>,
    mut placement_config: ResMut<TrackPlacementConfig>,
    mut commands: Commands,
) {
    let Some(from) = placement_config.initial_position else {
        debug!("Setting initial position to {:?}", cursor_position.0);
        placement_config.initial_position = Some(cursor_position.0);
        return;
    };
    let to = cursor_position.0;

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
        },
    ));

    placement_config.initial_position = None;
}
