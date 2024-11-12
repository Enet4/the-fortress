use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    ui::FocusPolicy,
    window::{WindowMode, WindowResolution},
};
use bevy_mod_picking::DefaultPickingPlugins;
use live::LiveActionPlugin;
use postprocess::PostProcessPlugin;
use scene::setup_scene;

mod effect;
mod live;
mod logic;
mod menu;
mod postprocess;
mod scene;
mod ui;

fn setup_ui(mut cmd: Commands) {
    // Node that fills entire background
    cmd.spawn(NodeBundle {
        focus_policy: FocusPolicy::Pass,
        style: Style {
            width: Val::Percent(100.),
            ..default()
        },
        ..default()
    })
    .with_children(|root| {
        // Text where we display the title
        root.spawn(TextBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_content: AlignContent::Center,
                ..default()
            },
            text: Text::from_section(
                "The Fortress",
                TextStyle {
                    font_size: 48.,
                    ..default()
                },
            ),
            ..default()
        });
    });
}

fn main() {
    App::new()
        .insert_resource(AmbientLight::NONE)
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "The Fortress".to_string(),
                        cursor: bevy::window::Cursor {
                            icon: CursorIcon::Crosshair,
                            visible: true,
                            ..Default::default()
                        },
                        fit_canvas_to_parent: true,
                        mode: WindowMode::Windowed,
                        resizable: true,
                        resolution: WindowResolution::new(1024., 768.),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(AssetPlugin {
                    // Never try to look up .meta files
                    meta_check: AssetMetaCheck::Never,
                    ..Default::default()
                }),
            PostProcessPlugin,
            LiveActionPlugin,
            DefaultPickingPlugins,
        ))
        .add_systems(Startup, (setup_scene, setup_ui))
        .add_systems(
            Update,
            (
                effect::apply_collapse,
                postprocess::oscillate_dithering,
                postprocess::fadeout_dithering,
            ),
        )
        .add_systems(PostUpdate, (effect::apply_glimmer, effect::apply_wobble))
        .run();
}
