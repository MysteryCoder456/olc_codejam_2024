use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

mod memory_bus;
mod process;

#[derive(Component)]
struct MainCamera;

#[derive(Resource)]
struct MousePosition(Vec2);

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
        .add_plugins((process::ProcessPlugin, memory_bus::MemoryBusPlugin))
        .insert_resource(MousePosition(Vec2::ZERO))
        .add_systems(Startup, setup)
        .add_systems(Update, update_mouse_position)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((MainCamera, Camera2dBundle::default()));
}

fn update_mouse_position(
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_pos: ResMut<MousePosition>,
    mut cursor_events: EventReader<CursorMoved>,
) {
    let (camera, camera_transform) = q_camera.single();

    for event in cursor_events.read() {
        mouse_pos.0 = camera
            .viewport_to_world_2d(camera_transform, event.position)
            .unwrap();
    }
}
