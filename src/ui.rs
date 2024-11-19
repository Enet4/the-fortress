//! Module for various common UI components
use bevy::{ecs::system::EntityCommands, prelude::*};

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

// button styles and utilities

const NORMAL_BUTTON: Color = Color::WHITE;
const HOVERED_BUTTON: Color = Color::srgb(0., 1., 1.);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0., 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.5, 0.5, 0.5);

pub fn spawn_button_with_style<'a, A: Component>(
    cmd: &'a mut ChildBuilder<'_>,
    text: impl Into<String>,
    style: Style,
    action: A,
) -> EntityCommands<'a> {
    let mut cmds = cmd.spawn((
        action,
        ButtonBundle {
            style,
            background_color: BackgroundColor(Color::BLACK),
            border_color: BorderColor(NORMAL_BUTTON),
            border_radius: BorderRadius::all(Val::Px(0.)),

            ..default()
        },
    ));
    cmds.with_children(|cmd| {
        cmd.spawn(TextBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font_size: 32.,
                    color: NORMAL_BUTTON,
                    ..default()
                },
            )
            .with_justify(JustifyText::Center),
            style: Style {
                margin: UiRect {
                    top: Val::Auto,
                    bottom: Val::Auto,
                    left: Val::Auto,
                    right: Val::Auto,
                },
                ..default()
            },
            ..default()
        });
    });
    cmds
}

pub fn spawn_button<'a, A: Component>(
    cmd: &'a mut ChildBuilder<'_>,
    text: impl Into<String>,
    action: A,
) -> EntityCommands<'a> {
    spawn_button_with_style(
        cmd,
        text,
        Style {
            width: Val::Auto,
            min_width: Val::Px(240.),
            border: UiRect::all(Val::Px(2.0)),
            padding: UiRect {
                top: Val::Px(10.),
                bottom: Val::Px(10.),
                left: Val::Px(20.),
                right: Val::Px(20.),
            },
            margin: UiRect::all(Val::Px(20.)),
            ..default()
        },
        action,
    )
}

#[derive(Debug, Component)]
pub struct SelectedOption;

/// generic button system
pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut border_color, selected) in &mut interaction_query {
        border_color.0 = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON,
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON,
            (Interaction::Hovered, None) => HOVERED_BUTTON,
            (Interaction::None, None) => NORMAL_BUTTON,
        }
    }
}
