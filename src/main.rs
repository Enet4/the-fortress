use assets::TextureHandles;
use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    ui::FocusPolicy,
    window::{WindowMode, WindowResolution},
};
use bevy_mod_picking::DefaultPickingPlugins;
use live::LiveActionPlugin;
use postprocess::PostProcessPlugin;

mod assets;
mod effect;
mod live;
mod logic;
mod menu;
mod postprocess;
mod structure;
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

/// All possible states in the game
#[derive(States, Default, Debug, Clone, Hash, Eq, PartialEq)]
pub enum AppState {
    Loading,
    /// The main part of the game
    #[default]
    Live,
    MainMenu,
}

fn main() {
    App::new()
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
        .add_systems(Startup, (live::setup_scene, setup_ui))
        .add_systems(
            Update,
            (
                effect::apply_collapse,
                postprocess::oscillate_dithering,
                postprocess::fadeout_dithering,
            ),
        )
        .add_systems(PostUpdate, (effect::apply_glimmer,))
        // add resources which we want to be able to load early
        .init_resource::<TextureHandles>()
        // add main state
        .init_state::<AppState>()
        .run();
}
