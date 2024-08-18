use bevy::{color::palettes::css::*, prelude::*};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{RegularPolygon, RegularPolygonFeature},
};

pub struct ProcessPlugin;
impl Plugin for ProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_process);
    }
}

#[derive(Component)]
struct Process;

fn spawn_process(mut commands: Commands) {
    let shape = RegularPolygon {
        sides: 6,
        feature: RegularPolygonFeature::Apothem(32.0),
        ..Default::default()
    };

    commands.spawn((
        Process,
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            ..Default::default()
        },
        Fill::color(DARK_GREEN.with_alpha(0.3)),
        Stroke::new(GREEN, 2.0),
    ));
}
