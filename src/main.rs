use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    window::WindowResolution,
};
use bevy_prototype_lyon::prelude::*;

mod process;

fn main() {
    let mut app = App::new();

    let log_level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    app.add_plugins(
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
    )
    .add_plugins(ShapePlugin)
    .add_plugins(process::ProcessPlugin)
    .add_systems(Startup, setup_app)
    .add_systems(PostStartup, spawn_test_processes);

    app.run();
}

fn setup_app(mut commands: Commands, mut clear_color: ResMut<ClearColor>) {
    commands.spawn(Camera2d::default());
    clear_color.0 = Color::BLACK;
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
