use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    window::WindowResolution,
};
use bevy_prototype_lyon::prelude::*;

mod process;
mod track;

#[derive(Resource)]
pub struct CursorPosition(Vec2);

fn main() {
    let mut app = App::new();

    let log_level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Oversimplified Computer Simulator".to_owned(),
                    resolution: WindowResolution::new(1152.0, 720.0),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(LogPlugin {
                level: log_level,
                ..Default::default()
            }),
        ShapePlugin,
    ))
    .add_plugins((process::ProcessPlugin, track::TrackPlugin))
    .insert_resource(CursorPosition(Vec2::ZERO))
    .add_systems(Startup, setup_app)
    .add_systems(PostStartup, spawn_test_processes)
    .add_systems(PreUpdate, update_cursor_position);

    app.run();
}

fn setup_app(mut commands: Commands, mut clear_color: ResMut<ClearColor>) {
    commands.spawn(Camera2d::default());
    clear_color.0 = Color::BLACK;
}

fn update_cursor_position(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut cursor_position: ResMut<CursorPosition>,
) {
    let (camera, camera_transform) = *camera_query;
    let window = *window;

    if let Some(pos) = window
        .cursor_position()
        .and_then(|pos| camera.viewport_to_world_2d(camera_transform, pos).ok())
    {
        cursor_position.0 = pos;
    }
}

fn spawn_test_processes(mut events: EventWriter<process::SpawnProcessEvent>) {
    events.send_batch([
        process::SpawnProcessEvent {
            position: Vec2::new(-100.0, 0.0),
        },
        process::SpawnProcessEvent {
            position: Vec2::new(80.0, 100.0),
        },
        process::SpawnProcessEvent {
            position: Vec2::new(120.0, -100.0),
        },
    ]);
}
