use crate::CursorPosition;
use bevy::{color::palettes::css::*, input::common_conditions::input_just_pressed, prelude::*};

pub struct TrackPlugin;

impl Plugin for TrackPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TrackPlacementConfig {
            initial_position: None,
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
}

fn draw_placement_indicator(
    cursor_position: Res<CursorPosition>,
    placement_config: Res<TrackPlacementConfig>,
    mut gizmos: Gizmos,
) {
    let Some(initial_position) = placement_config.initial_position else {
        return;
    };

    let color = NAVAJO_WHITE;
    let diff = cursor_position.0 - initial_position;

    let corner = if diff.x >= diff.y {
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
) {
    if let Some(initial_position) = placement_config.initial_position {
        debug!(
            "Placed track from {:?} to {:?}",
            initial_position, cursor_position.0
        );
        // spawn_track(initial_position, cursor_position.0);
        placement_config.initial_position = None;
    } else {
        debug!("Setting initial position to {:?}", cursor_position.0);
        placement_config.initial_position = Some(cursor_position.0);
    }
}
