use bevy::{color::palettes::css::*, prelude::*};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{RegularPolygon, RegularPolygonFeature},
};

#[derive(Event)]
pub struct SpawnProcessEvent {
    pub position: Vec2,
}

pub struct ProcessPlugin;

impl Plugin for ProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnProcessEvent>().add_systems(
            Update,
            spawn_processes.run_if(on_event::<SpawnProcessEvent>),
        );
    }
}

#[derive(Component)]
struct Process;

fn spawn_processes(mut commands: Commands, mut events: EventReader<SpawnProcessEvent>) {
    let shape = RegularPolygon {
        sides: 6,
        center: Vec2::ZERO,
        feature: RegularPolygonFeature::Radius(24.0),
    };

    for event in events.read() {
        debug!("Spawning process at {:?}", event.position);
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                transform: Transform::from_translation(event.position.extend(10.0)),
                ..Default::default()
            },
            Fill::color(GREEN),
            Stroke::new(DARK_GREEN, 4.0),
            Process,
        ));
    }
}
