use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use process::ProcessPlugin;

mod process;

fn main() {
    App::new()
        // Vendor plugins
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "RUN".to_owned(),
                    mode: bevy::window::WindowMode::Windowed,
                    resizable: true,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ShapePlugin,
        ))
        // In-house plugins
        .add_plugins(ProcessPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
