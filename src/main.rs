use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RUN".to_owned(),
                mode: bevy::window::WindowMode::Windowed,
                resizable: true,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .run();
}
