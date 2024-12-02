use bevy::{prelude::*, window::WindowResolution};

fn main() {
    println!("Hello, world!");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Oversimplified Computer Simulator".to_owned(),
                resolution: WindowResolution::new(1152.0, 720.0),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, setup_app)
        .run();
}

fn setup_app(mut commands: Commands, mut clear_color: ResMut<ClearColor>) {
    commands.spawn(Camera2d::default());
    clear_color.0 = Color::BLACK;
}
