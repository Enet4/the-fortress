use assets::TextureHandles;
use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    window::{WindowMode, WindowResolution},
};
use bevy_mod_picking::DefaultPickingPlugins;
use cheat::{Cheats, TextBuffer};
use live::LiveActionPlugin;
use menu::MenuPlugin;
use postprocess::PostProcessPlugin;

mod assets;
mod cheat;
mod effect;
mod live;
mod logic;
mod menu;
mod postprocess;
mod structure;
mod ui;

/// All possible states in the game
#[derive(States, Default, Debug, Clone, Hash, Eq, PartialEq)]
pub enum AppState {
    /// Some kind of splash screen for when the game is loading
    Loading,
    /// The main part of the game
    Live,
    /// The menu screen
    #[default]
    Menu,
}

/// Global game settings
#[derive(Debug, Resource)]
pub struct GameSettings {
    /// whether to show the amount of time the player is taking
    show_timer: bool,
    /// whether to enable sound
    sound: bool,
    /// whether to skip interludes
    /// (it will not skip the ones ending the game at the end of the sequence)
    skip_interludes: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            show_timer: false,
            sound: true,
            skip_interludes: false,
        }
    }
}

/// Marker for the main camera
#[derive(Component)]
pub struct CameraMarker;

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
            MenuPlugin,
            DefaultPickingPlugins,
        ))
        // systems which apply anywhere in the game
        .add_systems(
            Update,
            (
                effect::apply_collapse,
                effect::scale_up,
                postprocess::oscillate_dithering,
                postprocess::fadeout_dithering,
                cheat::cheat_input,
            ),
        )
        .add_systems(PostUpdate, (effect::apply_glimmer,))
        // add resources which are used globally
        .init_resource::<GameSettings>()
        .init_resource::<Cheats>()
        .init_resource::<TextBuffer>()
        // add resources which we want to be able to load early
        .init_resource::<TextureHandles>()
        // add main state
        .init_state::<AppState>()
        .run();
}

pub fn despawn_all_at<T: Component>(mut cmd: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}
