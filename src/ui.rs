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

/// Queries a specific meter and updates it to the given percentage.
/// This is a function meant to be used within a system.
#[inline]
pub fn set_meter_value<T>(mut q: Query<&mut Style, (With<Meter>, With<T>)>, percent: f32)
where
    T: Component,
{
    for mut style in q.iter_mut() {
        style.width = Val::Percent(percent);
    }
}
