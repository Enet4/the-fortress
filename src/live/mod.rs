//! The live action module, containing the active game logic
use std::fmt;

use bevy::{prelude::*, time::Stopwatch, ui::FocusPolicy};
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::*,
};

pub mod collision;
mod icon;
mod interlude;
mod levels;
mod mob;
pub mod obstacle;
mod phase;
mod player;
mod projectile;
mod scene;
mod weapon;

use interlude::AdvanceInterlude;
use levels::CurrentLevel;
use mob::MobSpawner;
use phase::PhaseTrigger;
use player::{
    process_attacks, process_damage_player, process_player_movement, update_player_cooldown_meter,
    update_player_health_meter, DamagePlayer, Player, PlayerMovement, TargetDestroyed,
};
use projectile::ProjectileAssets;
use weapon::{ChangeWeapon, PlayerAttack, WeaponCubeAssets};
// re-export some stuff
pub use weapon::TriggerWeapon;

use crate::{
    assets::{AudioHandles, DefaultFont},
    despawn_all_at,
    effect::{
        self, apply_collapse, apply_rotation, apply_velocity, stay_on_floor, time_to_live,
        Collapsing,
    },
    logic::{Num, TargetRule},
    structure::Fork,
    ui::{button_system, spawn_button_in_group, spawn_button_with_style, MeterBundle},
    AppState, GameSettings,
};

use super::CameraMarker;

/// Running or paused
#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(AppState = AppState::Live)]
enum LiveState {
    /// Intermediate state for loading a new level
    #[default]
    LoadingLevel,
    /// Running
    Running,
    /// On pause screen
    Paused,
    /// Showing an interlude message
    ShowingInterlude,
    /// Defeat screen
    Defeat,
}

/// The plugin which adds everything related to the live action
pub struct LiveActionPlugin;

impl Plugin for LiveActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_ui_anchor::AnchorUiPlugin::<CameraMarker>::new())
            // game states
            .init_state::<LiveState>()
            // live game setup
            .add_systems(
                OnEnter(AppState::Live),
                (scene::setup_scene, setup_ui, start_running).chain(),
            )
            // partial live game take-down when exiting Running and entering Loading
            .add_systems(
                OnTransition {
                    exited: LiveState::Running,
                    entered: LiveState::LoadingLevel,
                },
                (
                    despawn_all_at::<OnLive>,
                    scene::setup_scene,
                    setup_ui,
                    start_running,
                )
                    .chain(),
            )
            // partial live game take-down when exiting Defeat and entering Loading
            .add_systems(
                OnTransition {
                    exited: LiveState::Defeat,
                    entered: LiveState::LoadingLevel,
                },
                despawn_all_at::<OnLive>,
            )
            // live game take-down
            .add_systems(
                OnExit(AppState::Live),
                (despawn_all_at::<OnLive>, reset_game),
            )
            .add_systems(OnEnter(LiveState::Defeat), enter_defeat)
            // systems which should function regardless of the game state
            .add_systems(Update, pause_on_esc.run_if(in_state(AppState::Live)))
            // systems that only run when the game is running
            .add_systems(
                Update,
                (
                    update_player_cooldown_meter,
                    update_player_health_meter,
                    effect::apply_wobble,
                    effect::fade_away,
                    effect::apply_rotation,
                    icon::update_icon_opacity,
                    weapon::update_cooldown,
                    weapon::weapon_keyboard_input,
                    weapon::weapon_button_action,
                    weapon::process_weapon_button_selected,
                    weapon::process_weapon_button_deselected,
                    (
                        process_player_movement,
                        apply_velocity,
                        apply_rotation,
                        stay_on_floor,
                    )
                        .chain(),
                    (
                        icon::clear_icons_of_destroyed_things,
                        apply_collapse,
                        time_to_live,
                        process_end_of_corridor,
                        mob::process_spawner_trigger,
                        interlude::process_interlude_trigger,
                        button_system::<Decision>,
                        decision_action,
                    )
                        .chain(),
                )
                    .run_if(in_state(LiveState::Running)),
            )
            // running at fixed step
            .add_systems(
                FixedUpdate,
                (
                    projectile::projectile_collision,
                    process_attacks,
                    process_target_destroyed,
                    process_new_target,
                    mob::spawn_mobs,
                    mob::destroy_spawner_when_done,
                    process_damage_player,
                    (process_live_time, update_timer_text).chain(),
                    weapon::process_weapon_change,
                    weapon::trigger_weapon,
                    weapon::process_new_weapon,
                    weapon::process_approach_weapon_cube,
                    phase::process_approach_dread,
                    phase::process_approach_move_on,
                    button_system::<weapon::WeaponButton>,
                    on_enter_next_level,
                )
                    .run_if(in_state(LiveState::Running)),
            )
            // paused
            .add_systems(
                Update,
                (button_system::<PauseButton>, paused_button_action)
                    .run_if(in_state(LiveState::Paused)),
            )
            // defeat
            .add_systems(
                Update,
                (
                    (button_system::<DefeatButton>, defeat_button_action),
                    (
                        // these effects are also OK in the defeat screen
                        effect::apply_wobble,
                        effect::fade_away,
                        effect::apply_rotation,
                        effect::apply_velocity,
                        stay_on_floor,
                        icon::update_icon_opacity,
                    )
                        .chain(),
                )
                    .run_if(in_state(LiveState::Defeat)),
            )
            // interlude
            .add_systems(
                Update,
                (
                    interlude::fade_in_interlude,
                    interlude::fade_out_interlude,
                    interlude::on_click_advance_interlude,
                    interlude::process_advance_interlude,
                )
                    .run_if(in_state(LiveState::ShowingInterlude)),
            )
            // resources
            .init_resource::<CurrentLevel>()
            .init_resource::<LiveTime>()
            .init_resource::<ProjectileAssets>()
            .init_resource::<WeaponCubeAssets>()
            .init_resource::<mob::MobAssets>()
            .insert_resource(AmbientLight::NONE)
            // events
            .add_event::<TriggerWeapon>()
            .add_event::<ChangeWeapon>()
            .add_event::<PlayerAttack>()
            .add_event::<TargetDestroyed>()
            .add_event::<DamagePlayer>()
            .add_event::<AdvanceInterlude>()
            .add_event::<AdvanceLevel>();
    }
}

fn start_running(mut next_state: ResMut<NextState<LiveState>>) {
    next_state.set(LiveState::Running);
}

fn reset_game(
    mut next_state: ResMut<NextState<LiveState>>,
    mut live_time: ResMut<LiveTime>,
    mut current_level: ResMut<CurrentLevel>,
) {
    next_state.set(LiveState::default());
    live_time.reset();
    current_level.reset();
}

fn enter_defeat(mut defeat_div_q: Query<&mut Style, With<DefeatDiv>>) {
    for mut style in defeat_div_q.iter_mut() {
        style.display = Display::Flex;
    }
}

/// Marker component for everything in live mode
#[derive(Debug, Default, Component)]
pub struct OnLive;

/// Resource that keeps track of the live (in-game) time.
///
/// With this one, time does not count while it is paused.
#[derive(Debug, Default, Resource)]
pub struct LiveTime(pub Stopwatch);

impl LiveTime {
    pub fn reset(&mut self) {
        self.0.reset();
    }

    pub fn elapsed_seconds(&self) -> f32 {
        self.0.elapsed_secs() as f32
    }
}

impl fmt::Display for LiveTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // mm:ss.SS
        let elapsed = self.0.elapsed_secs();
        let elapsed_whole = elapsed as i64;
        let minutes = elapsed_whole / 60;
        let rest = elapsed - (minutes as f32 * 60.);
        write!(f, "{minutes:02}:{rest:05.2}")
    }
}

fn process_live_time(time: Res<Time>, mut live_time: ResMut<LiveTime>) {
    live_time.0.tick(time.delta());
}

/// pause the game when the player presses the escape key
fn pause_on_esc(
    input: Res<ButtonInput<KeyCode>>,
    paused_state: Res<State<LiveState>>,
    mut next_paused_state: ResMut<NextState<LiveState>>,
    mut paused_node_q: Query<&mut Style, With<PausedDiv>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        match paused_state.get() {
            LiveState::Running => {
                next_paused_state.set(LiveState::Paused);
                for mut style in paused_node_q.iter_mut() {
                    style.display = Display::Flex;
                }
                println!("Game paused");
            }
            LiveState::Paused => {
                next_paused_state.set(LiveState::Running);
                for mut style in paused_node_q.iter_mut() {
                    style.display = Display::None;
                }
                println!("Game resumed");
            }
            LiveState::LoadingLevel | LiveState::ShowingInterlude | LiveState::Defeat => {
                // ignore
            }
        }
    }
}

/// Component for things with a health meter.
///
/// Most attacks will deduct `1.` from health,
/// and most health meters will have a maximum of `1.`.
///
/// The player will have some more health than the mobs.
#[derive(Debug, Component)]
pub struct Health {
    pub value: f32,
    pub max: f32,
}

impl Health {
    pub fn new(hp: f32) -> Self {
        Self { value: hp, max: hp }
    }

    /// Reset health to its maximum
    pub fn replenish(&mut self) {
        self.value = self.max;
    }
}

impl Default for Health {
    fn default() -> Self {
        Health::new(1.)
    }
}

/// Component for anything that is an attack target to the player.
#[derive(Debug, Default, Component)]
pub struct Target {
    /// the number affecting how the target should be attacked
    pub num: Num,
    /// the rule for attacking the target
    pub rule: TargetRule,
}

/// Component for the player's attack cooldown meter
#[derive(Debug, Default, Component)]
pub struct CooldownMeter;

/// Component for the player's health meter
#[derive(Debug, Default, Component)]
pub struct HealthMeter;

/// Marker component for a UI node containing the weapon selectors
#[derive(Debug, Default, Component)]
pub struct WeaponListNode;

/// Marker component for the UI node that shows the game is paused
#[derive(Debug, Default, Component)]
struct PausedDiv;

/// Group marker component for the buttons in the paused game screen
#[derive(Debug, Default, Component)]
struct PauseButton;

#[derive(Debug, Component)]
enum PausedButtonAction {
    Resume,
    GiveUp,
}

/// Marker component for the UI node that shows defeat
#[derive(Debug, Default, Component)]
struct DefeatDiv;

/// Group marker component for the buttons in the defeat screen
#[derive(Debug, Default, Component)]
struct DefeatButton;

#[derive(Debug, Component)]
enum DefeatButtonAction {
    Restart,
    GiveUp,
}

/// Marker component for the text entity showing the game timer.
#[derive(Debug, Component)]
pub struct TimeIndicator;

/// Set up the main UI components in the game for the first time
fn setup_ui(mut cmd: Commands, default_font: Res<DefaultFont>, game_settings: Res<GameSettings>) {
    let font = &default_font.0;

    // Node for the bottom HUD
    cmd.spawn((
        OnLive,
        NodeBundle {
            focus_policy: FocusPolicy::Pass,
            style: Style {
                display: Display::Flex,
                bottom: Val::Px(0.),
                align_self: AlignSelf::FlexEnd,
                width: Val::Percent(100.),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::FlexEnd,
                ..default()
            },
            ..default()
        },
    ))
    .with_children(|root| {
        root.spawn((
            WeaponListNode,
            NodeBundle {
                style: Style {
                    margin: UiRect {
                        bottom: Val::Px(4.),
                        top: Val::Auto,
                        left: Val::Auto,
                        right: Val::Auto,
                    },
                    ..default()
                },
                ..default()
            },
        ));

        // if enabled, add timer indicator
        if game_settings.show_timer {
            root.spawn((
                TimeIndicator,
                TextBundle {
                    text: Text::from_section(
                        "00:00.00",
                        TextStyle {
                            color: Color::WHITE,
                            font: font.clone(),
                            font_size: 24.,
                            ..default()
                        },
                    ),
                    focus_policy: FocusPolicy::Pass,
                    style: Style {
                        left: Val::Px(4.),
                        top: Val::Px(4.),
                        bottom: Val::Px(4.),
                        right: Val::Auto,
                        ..default()
                    },
                    ..default()
                },
            ));
        }

        // insert cooldown meter
        root.spawn((
            MeterBundle::new(Val::Px(10.), Color::srgba_u8(0, 63, 255, 192)),
            CooldownMeter,
        ));

        // insert health meter
        root.spawn((
            MeterBundle::new(Val::Px(42.), Color::srgba_u8(0, 224, 7, 192)),
            HealthMeter,
        ));
    });

    // node for the pausing screen, which is hidden by default
    cmd.spawn((
        PausedDiv,
        OnLive,
        NodeBundle {
            style: Style {
                display: Display::None,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            z_index: ZIndex::Global(10),
            background_color: BackgroundColor(Color::srgba(0., 0., 0., 0.5)),
            ..default()
        },
    ))
    .with_children(|cmd| {
        // button to resume the game
        spawn_button_in_group(
            cmd,
            font.clone(),
            "Resume",
            PauseButton,
            PausedButtonAction::Resume,
        );

        // button to return to main menu
        spawn_button_in_group(
            cmd,
            font.clone(),
            "Give Up",
            PauseButton,
            PausedButtonAction::GiveUp,
        );
    });

    // node for the defeat screen, which is also hidden by default
    cmd.spawn((
        DefeatDiv,
        OnLive,
        NodeBundle {
            style: Style {
                display: Display::None,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            z_index: ZIndex::Global(10),
            background_color: BackgroundColor(Color::srgba(1., 0., 0., 0.25)),
            ..default()
        },
    ))
    .with_children(|cmd| {
        cmd.spawn(TextBundle {
            style: Style {
                margin: UiRect {
                    bottom: Val::Px(32.),
                    ..default()
                },
                ..default()
            },
            text: Text::from_section(
                "Try Again?",
                TextStyle {
                    color: Color::srgb(0.85, 0.85, 0.85),
                    font: font.clone(),
                    font_size: 32.,
                    ..default()
                },
            ),
            ..default()
        });

        // button to restart the current level
        spawn_button_in_group(
            cmd,
            font.clone(),
            "Restart Level",
            DefeatButton,
            DefeatButtonAction::Restart,
        );

        // button to return to main menu
        spawn_button_in_group(
            cmd,
            font.clone(),
            "Give Up",
            DefeatButton,
            DefeatButtonAction::GiveUp,
        );
    });
}

/// system that updates the timer indicator with the time passed
fn update_timer_text(
    live_time: Res<LiveTime>,
    mut time_text_q: Query<&mut Text, With<TimeIndicator>>,
) {
    for mut time_text in &mut time_text_q {
        let Some(section) = time_text.sections.get_mut(0) else {
            continue;
        };

        section.value = live_time.to_string();
    }
}

/// system which handles button presses in the paused screen
fn paused_button_action(
    mut cmd: Commands,
    mut interaction_query: Query<
        (&Interaction, &PausedButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut paused_node_q: Query<&mut Style, With<PausedDiv>>,
    mut live_state: ResMut<NextState<LiveState>>,
    mut game_state: ResMut<NextState<AppState>>,
    audio_handles: Res<AudioHandles>,
) {
    for (interaction, pause_button_action) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            audio_handles.play_zipclick(&mut cmd);
            match pause_button_action {
                PausedButtonAction::Resume => {
                    for mut style in paused_node_q.iter_mut() {
                        style.display = Display::None;
                    }
                    live_state.set(LiveState::Running);
                    println!("Game resumed");
                }
                PausedButtonAction::GiveUp => {
                    // return to main menu
                    game_state.set(AppState::Menu);
                    println!("Giving up...");
                }
            }
        }
    }
}

/// system which handles button presses in the defeat screen
fn defeat_button_action(
    mut cmd: Commands,
    mut interaction_query: Query<
        (&Interaction, &DefeatButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut defeat_node_q: Query<&mut Style, With<DefeatDiv>>,
    mut live_state: ResMut<NextState<LiveState>>,
    mut game_state: ResMut<NextState<AppState>>,
    audio_handles: Res<AudioHandles>,
) {
    for (interaction, pause_button_action) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            audio_handles.play_zipclick(&mut cmd);
            match pause_button_action {
                DefeatButtonAction::Restart => {
                    let Ok(mut defeat_node_style) = defeat_node_q.get_single_mut() else {
                        break;
                    };
                    defeat_node_style.display = Display::None;
                    live_state.set(LiveState::LoadingLevel);
                }
                DefeatButtonAction::GiveUp => {
                    // return to main menu
                    game_state.set(AppState::Menu);
                }
            }
        }
        break;
    }
}

/// general system callback for when the player clicks on something
pub fn callback_on_click(event: Listener<Pointer<Click>>, mut events: EventWriter<TriggerWeapon>) {
    if event.button != PointerButton::Primary {
        return;
    }
    let Some(target_pos) = event.hit.position.clone() else {
        return;
    };

    events.send(TriggerWeapon { target_pos });
}

/// a system to handle game state changes when a target is destroyed
pub fn process_target_destroyed(
    mut target_destroyed_events: EventReader<TargetDestroyed>,
    active_mob_spawners_q: Query<Entity, (With<MobSpawner>, Without<PhaseTrigger>)>,
    target_q: Query<Entity, (With<Target>, Without<Collapsing>)>,
    mut player_q: Query<&mut PlayerMovement, With<Player>>,
) {
    let mut done = false;
    for _ in target_destroyed_events.read() {
        println!("Target destroyed");
        if done {
            // if done, we can consume the rest of the events and continue normally
            continue;
        }
        // count the number of targets still on scene
        let num_targets = target_q.iter().count();
        if num_targets > 0 {
            continue;
        }
        // and count the number of mob spawners still on scene
        let num_mobspawners = active_mob_spawners_q.iter().count();
        if num_mobspawners > 0 {
            continue;
        }

        println!("No more activity on screen");
        // let's move!
        let mut player_movement = player_q.single_mut();
        *player_movement = PlayerMovement::Walking;
        done = true;
    }
}

/// system to stop the player if new targets emerge
pub fn process_new_target(
    target_q: Query<(Entity, &Target), Added<Target>>,
    mut player_q: Query<&mut PlayerMovement, With<Player>>,
) {
    if target_q.iter().count() == 0 {
        return;
    }

    let mut player_movement = player_q.single_mut();
    *player_movement = PlayerMovement::Halting;
}

/// system detecting that the player has reached the end of the corridor
pub fn process_end_of_corridor(
    mut cmd: Commands,
    mut player_q: Query<
        (&mut PlayerMovement, &mut Health, &Transform),
        (With<Player>, Changed<Transform>),
    >,
    fork_q: Query<&Transform, With<Fork>>,
    default_font: Res<DefaultFont>,
) {
    // retrieve player
    let Ok((mut player_movement, mut health, player_transform)) = player_q.get_single_mut() else {
        return;
    };

    // retrieve the fork
    let Ok(fork_transform) = fork_q.get_single() else {
        return;
    };

    let player_pos = player_transform.translation;
    if player_pos.z + 13. >= fork_transform.translation.z {
        // stop walking
        *player_movement = PlayerMovement::Idle;

        // heal player
        health.replenish();

        // and spawn new input arrows to select which way to go
        spawn_decision_arrows(&mut cmd, default_font);
    }
}

/// Marker component for the UI node containing the decision arrows
#[derive(Debug, Component)]
struct DecisionArrowsDiv;

#[derive(Debug, Copy, Clone, PartialEq, Component)]
enum Decision {
    Left,
    Right,
}

fn spawn_decision_arrows(cmd: &mut Commands, default_font: Res<DefaultFont>) {
    let font = &default_font.0;
    cmd.spawn((
        OnLive,
        DecisionArrowsDiv,
        NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Stretch,
                width: Val::Percent(100.),
                column_gap: Val::Auto,
                height: Val::Auto,
                margin: UiRect {
                    top: Val::Auto,
                    bottom: Val::Auto,
                    left: Val::Auto,
                    right: Val::Auto,
                    ..default()
                },
                ..default()
            },
            ..default()
        },
    ))
    .with_children(|cmd| {
        spawn_button_with_style(
            cmd,
            font.clone(),
            "<",
            Style {
                width: Val::Px(200.),
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
            Decision::Left,
        );
        spawn_button_with_style(
            cmd,
            font.clone(),
            ">",
            Style {
                width: Val::Px(200.),
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
            Decision::Right,
        );
    });
}

/// system that handles the choice of the player
fn decision_action(
    mut cmd: Commands,
    mut interaction_query: Query<(&Interaction, &Decision), (Changed<Interaction>, With<Button>)>,
    mut advance_level_events: EventWriter<AdvanceLevel>,
    audio_handles: Res<AudioHandles>,
) {
    for (interaction, decision) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // play sound
            audio_handles.play_zipclick(&mut cmd);

            advance_level_events.send(AdvanceLevel(*decision));
            break;
        }
    }
}

/// Event for when the game will advance to the next level
#[derive(Debug, Event)]
struct AdvanceLevel(Decision);

fn on_enter_next_level(
    mut events: EventReader<AdvanceLevel>,
    mut current_level: ResMut<CurrentLevel>,
    mut next_state: ResMut<NextState<LiveState>>,
) {
    for AdvanceLevel(decision) in events.read() {
        current_level.advance(*decision);
        next_state.set(LiveState::LoadingLevel);
        break;
    }
}
