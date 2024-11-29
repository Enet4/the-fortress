//! Module for various common UI components
use bevy::{ecs::system::EntityCommands, prelude::*};

/// Resource for the sizes to use in most common UI components.
#[derive(Debug, Resource)]
pub struct Sizes {
    pub button_min_width: f32,
    pub title_font_size: f32,
    pub button_font_size: f32,
    pub interlude_font_size: f32,
    pub outer_padding: f32,
}

impl Default for Sizes {
    fn default() -> Self {
        Sizes {
            button_min_width: 240.,
            title_font_size: 72.,
            button_font_size: 40.,
            interlude_font_size: 28.,
            outer_padding: 14.,
        }
    }
}

impl Sizes {
    /// Sizes for a smaller screen
    pub const SMALL: Self = Sizes {
        button_min_width: 180.,
        title_font_size: 46.,
        button_font_size: 25.,
        interlude_font_size: 20.,
        outer_padding: 2.,
    };
}

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

fn spawn_button_impl<'a, A, G>(
    cmd: &'a mut ChildBuilder<'_>,
    sizes: &Sizes,
    font: Handle<Font>,
    text: impl Into<String>,
    style: Option<Style>,
    group: Option<G>,
    action: A,
) -> EntityCommands<'a>
where
    A: Component,
    G: Component,
{
    let style = style.unwrap_or_else(|| Style {
        width: Val::Auto,
        min_width: Val::Px(sizes.button_min_width),
        border: UiRect::all(Val::Px(2.0)),
        padding: UiRect::axes(Val::Px(16.), Val::Px(8.)),
        margin: UiRect::all(Val::Px(14.)),
        ..default()
    });

    let bundle = (
        action,
        ButtonBundle {
            style,
            background_color: BackgroundColor(Color::BLACK),
            border_color: BorderColor(NORMAL_BUTTON),
            border_radius: BorderRadius::ZERO,
            ..default()
        },
    );
    let mut cmds = if let Some(group) = group {
        cmd.spawn((bundle, group))
    } else {
        cmd.spawn(bundle)
    };

    cmds.with_children(|cmd| {
        cmd.spawn(TextBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font,
                    font_size: sizes.button_font_size,
                    color: NORMAL_BUTTON,
                    ..default()
                },
            )
            .with_justify(JustifyText::Center),
            style: Style {
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            ..default()
        });
    });
    cmds
}

pub fn spawn_button_with_style<'a, A>(
    cmd: &'a mut ChildBuilder<'_>,
    sizes: &Sizes,
    font: Handle<Font>,
    text: impl Into<String>,
    style: Style,
    action: A,
) -> EntityCommands<'a>
where
    A: Component,
{
    spawn_button_impl(cmd, sizes, font, text, Some(style), None::<Button>, action)
}

pub fn spawn_button_in_group_with_style<'a, A, G>(
    cmd: &'a mut ChildBuilder<'_>,
    sizes: &Sizes,
    font: Handle<Font>,
    text: impl Into<String>,
    style: Style,
    group: G,
    action: A,
) -> EntityCommands<'a>
where
    A: Component,
    G: Component,
{
    spawn_button_impl(cmd, sizes, font, text, Some(style), Some(group), action)
}

pub fn spawn_button_in_group<'a, A, G>(
    cmd: &'a mut ChildBuilder<'_>,
    sizes: &Sizes,
    font: Handle<Font>,
    text: impl Into<String>,
    group: G,
    action: A,
) -> EntityCommands<'a>
where
    A: Component,
    G: Component,
{
    spawn_button_impl(cmd, sizes, font, text, None, Some(group), action)
}

/// Spawn a button, no group, default styles
#[inline]
pub fn spawn_button<'a, A>(
    cmd: &'a mut ChildBuilder<'_>,
    sizes: &Sizes,
    font: Handle<Font>,
    text: impl Into<String>,
    action: A,
) -> EntityCommands<'a>
where
    A: Component,
{
    spawn_button_impl(cmd, sizes, font, text, None, None::<Button>, action)
}

#[derive(Debug, Component)]
pub struct SelectedOption;

/// generic button system for updating a button style
/// (use `T` for a specific button marker group,
/// use `Button` if this isn't important)
pub fn button_system<T>(
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<T>),
    >,
) where
    T: Component,
{
    for (interaction, mut border_color, selected) in &mut interaction_query {
        border_color.0 = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON,
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON,
            (Interaction::Hovered, None) => HOVERED_BUTTON,
            (Interaction::None, None) => NORMAL_BUTTON,
        }
    }
}

pub fn update_buttons_on_window_resize(
    sizes: Res<Sizes>,
    mut button_q: Query<(&mut Style, &Children), With<Button>>,
    mut button_text_q: Query<&mut Text>,
) {
    if !sizes.is_changed() && !sizes.is_added() {
        return;
    }
    for (mut style, children) in &mut button_q {
        style.min_width = Val::Px(sizes.button_min_width);

        // update the button text font size
        let font_size = sizes.button_font_size;
        for child in children {
            let Ok(mut text) = button_text_q.get_mut(*child) else {
                continue;
            };
            // update all sections
            for section in &mut text.sections {
                section.style.font_size = font_size;
            }
        }
    }
}
