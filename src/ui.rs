//! Module for various important UI components
use bevy::prelude::*;

#[derive(Debug, Default, Component)]
pub struct Meter;

/// A rectangle of fixed height
/// that fills up with a color from 0% to 100% width
/// based on a meter value.
#[derive(Debug, Default, Bundle)]
pub struct MeterBundle {
    pub meter: Meter,
    #[bundle()]
    pub rect: NodeBundle,
}

impl MeterBundle {
    pub fn new(height: Val, fill_color: Color) -> Self {
        MeterBundle {
            rect: NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height,
                    ..default()
                },
                background_color: BackgroundColor(fill_color),
                ..default()
            },
            ..default()
        }
    }
}

pub fn update_meter<T, F>(mut q: Query<(&mut Style, &T), With<Meter>>)
where
    T: Component,
    F: Default + Fn(&T) -> f32,
{
    for (mut style, meter) in q.iter_mut() {
        style.width = Val::Percent(F::default()(meter));
    }
}
