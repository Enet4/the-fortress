use bevy::{prelude::*, window::WindowMode};
use live::LiveActionPlugin;
use postprocess::PostProcessPlugin;
use scene::setup_scene;

mod effect;
mod live;
mod postprocess;
mod scene;

fn setup_ui(mut cmd: Commands) {
    // Node that fills entire background
    cmd.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            ..default()
        },
        ..default()
    })
    .with_children(|root| {
        // Text where we display current resolution
        root.spawn((TextBundle::from_section(
            "The Fortress",
            TextStyle {
                font_size: 32.0,
                ..default()
            },
        ),));
    });
}

fn main() {
    App::new()
        .insert_resource(AmbientLight::NONE)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "The Fortress".to_string(),
                    cursor: bevy::window::Cursor {
                        icon: CursorIcon::Crosshair,
                        visible: true,
                        ..Default::default()
                    },
                    fit_canvas_to_parent: true,
                    mode: WindowMode::Windowed,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            PostProcessPlugin,
            LiveActionPlugin,
        ))
        .add_systems(Startup, (setup_scene, setup_ui))
        .add_systems(
            Update,
            (postprocess::update_settings, postprocess::fadeout_dithering),
        )
        .add_systems(PostUpdate, (effect::apply_glimmer, effect::apply_wobble))
        .run();
}
