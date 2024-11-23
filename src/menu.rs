//! Components and systems for the main menu

use bevy::prelude::*;

use crate::{
    despawn_all_at,
    ui::{button_system, spawn_button},
    AppState, CameraMarker, GameSettings,
};

#[derive(SubStates, Debug, Default, Clone, Eq, Hash, PartialEq)]
#[source(AppState = AppState::Menu)]
enum MenuState {
    /// Initializing the main menu
    #[default]
    Init,
    /// The main menu root
    Main,
    /// A separate section for the settings screen
    Settings,
    /// Disabled
    Disabled,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>()
            .add_systems(OnEnter(AppState::Menu), menu_setup)
            .add_systems(OnEnter(MenuState::Main), main_menu_setup)
            .add_systems(OnExit(MenuState::Main), despawn_all_at::<OnMainMenu>)
            .add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
            .add_systems(
                OnExit(MenuState::Settings),
                despawn_all_at::<OnSettingsMenu>,
            )
            .add_systems(OnExit(AppState::Menu), despawn_all_at::<MenuScreen>)
            .add_systems(
                Update,
                (menu_action, button_system::<Button>).run_if(in_state(AppState::Menu)),
            );
    }
}

#[derive(Debug, Component)]
enum MenuButtonAction {
    // - main -
    Start,
    Settings,
    Exit,
    // - options -
    ToggleSound,
    ToggleTimer,
    /// return to main menu
    BackToMainMenu,
}

/// Marker component for the full menu screen
#[derive(Debug, Component)]
struct MenuScreen;

/// system to set up the menu UI (applies to all menu sections)
fn menu_setup(mut cmd: Commands, mut next_state: ResMut<NextState<MenuState>>) {
    // Title
    cmd.spawn((
        MenuScreen,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                ..default()
            },
            ..default()
        },
    ))
    .with_children(|root| {
        // Text where we display the title
        root.spawn(TextBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_content: AlignContent::Center,
                margin: UiRect {
                    top: Val::Px(16.),
                    left: Val::Auto,
                    right: Val::Auto,
                    ..default()
                },
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

    // Camera
    cmd.spawn((
        MenuScreen,
        CameraMarker,
        IsDefaultUiCamera,
        Camera2dBundle::default(),
    ));

    next_state.set(MenuState::Main);
}

#[derive(Debug, Component)]
pub struct OnMainMenu;

/// system to spawn the main menu UI
pub fn main_menu_setup(mut cmd: Commands) {
    // division for main buttons
    cmd.spawn((
        OnMainMenu,
        NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.),
                margin: UiRect {
                    top: Val::Auto,
                    bottom: Val::Auto,
                    ..default()
                },
                ..default()
            },
            ..default()
        },
    ))
    .with_children(|cmd| {
        // button to start the game
        spawn_button(cmd, "Start", MenuButtonAction::Start);
        // open options
        spawn_button(cmd, "Settings", MenuButtonAction::Settings);
        // button to exit the game
        spawn_button(cmd, "Exit", MenuButtonAction::Exit);
    });
}

#[derive(Debug, Component)]
pub struct OnSettingsMenu;

/// system to spawn the main menu UI
pub fn settings_menu_setup(mut cmd: Commands) {
    // division for main buttons
    cmd.spawn((
        OnSettingsMenu,
        NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.),
                margin: UiRect {
                    top: Val::Auto,
                    bottom: Val::Auto,
                    ..default()
                },
                ..default()
            },
            ..default()
        },
    ))
    .with_children(|cmd| {
        spawn_button(cmd, "Show Timer: OFF", MenuButtonAction::ToggleTimer);
        // open options
        spawn_button(cmd, "Sound: ON", MenuButtonAction::ToggleSound);
        // button to exit the game
        spawn_button(cmd, "Back", MenuButtonAction::BackToMainMenu);
    });
}

fn menu_action(
    mut interaction_query: Query<
        (&Interaction, &MenuButtonAction, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<AppState>>,
    mut settings: ResMut<GameSettings>,
    mut button_text_q: Query<&mut Text>,
) {
    for (interaction, menu_button_action, children) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Exit => {
                    app_exit_events.send(AppExit::Success);
                }
                MenuButtonAction::Start => {
                    game_state.set(AppState::Live);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),

                MenuButtonAction::ToggleSound => {
                    settings.sound = !settings.sound;
                    let new_text = if settings.sound {
                        "Sound: ON"
                    } else {
                        "Sound: OFF"
                    };
                    for child in children {
                        if let Ok(mut text) = button_text_q.get_mut(*child) {
                            text.sections[0].value = new_text.to_string();
                        }
                    }
                }
                MenuButtonAction::ToggleTimer => {
                    settings.show_timer = !settings.show_timer;
                    let new_text = if settings.show_timer {
                        "Show Timer: ON"
                    } else {
                        "Show Timer: OFF"
                    };
                    for child in children {
                        if let Ok(mut text) = button_text_q.get_mut(*child) {
                            text.sections[0].value = new_text.to_string();
                        }
                    }
                }
            }
        }
    }
}
