use crate::BusStop;
use bevy::{color::palettes::css::*, prelude::*};
use bevy_prototype_lyon::{
    prelude::*,
    shapes::{Rectangle, RegularPolygon, RegularPolygonFeature},
};

#[derive(Event)]
pub struct SpawnBusEvent;

#[derive(Event)]
pub struct SpawnBusStationEvent {
    pub position: Vec2,
}

pub struct BusPlugin;

impl Plugin for BusPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnBusEvent>()
            .add_event::<SpawnBusStationEvent>()
            .add_systems(
                Update,
                (
                    spawn_bus, // FIXME: doesn't run at all with this condition applied: .run_if(on_event::<SpawnBusEvent>),
                    spawn_bus_station.run_if(on_event::<SpawnBusStationEvent>),
                ),
            );
    }
}

#[derive(Component)]
struct Bus;

#[derive(Component)]
struct BusStation;

fn spawn_bus(
    mut commands: Commands,
    mut events: EventReader<SpawnBusEvent>,
    station_query: Single<&Transform, With<BusStation>>,
) {
    let station_tf = *station_query;
    let shape = Rectangle {
        extents: Vec2::new(24.0, 16.0),
        origin: RectangleOrigin::Center,
        radii: Some(BorderRadii::single(2.0)),
    };

    for _event in events.read() {
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                transform: Transform::from_translation(station_tf.translation.with_z(20.0)),
                ..Default::default()
            },
            Fill::color(ORANGE),
            Bus,
        ));
    }
}

fn spawn_bus_station(mut commands: Commands, mut events: EventReader<SpawnBusStationEvent>) {
    let shape = RegularPolygon {
        sides: 6,
        center: Vec2::ZERO,
        feature: RegularPolygonFeature::Radius(24.0),
    };

    for event in events.read() {
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                transform: Transform::from_translation(event.position.extend(10.0)),
                ..Default::default()
            },
            Fill::color(ORANGE_RED),
            Stroke::new(ORANGE_RED.darker(0.1), 4.0),
            BusStation,
            BusStop,
        ));
    }
}
