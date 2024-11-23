//! The live action module, containing the active game logic
use bevy::{prelude::*, time::Stopwatch, ui::FocusPolicy};
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::*,
};

pub mod collision;
mod icon;
mod interlude;
mod mob;
pub mod obstacle;
mod phase;
mod player;
mod projectile;
mod scene;
mod weapon;

use interlude::AdvanceInterlude;
use player::{
    process_attacks, process_damage_player, process_player_movement, update_player_cooldown_meter,
    update_player_health_meter, DamagePlayer, Player, PlayerMovement, TargetDestroyed,
};
use projectile::ProjectileAssets;
use weapon::{ChangeWeapon, PlayerAttack, WeaponCubeAssets};
// re-export some stuff
pub use weapon::TriggerWeapon;

use crate::{
    despawn_all_at,
    effect::{
        self, apply_collapse, apply_rotation, apply_velocity, stay_on_floor, time_to_live,
        Collapsing,
    },
    logic::{Num, TargetRule},
    structure::Fork,
    ui::{button_system, spawn_button_in_group, spawn_button_with_style, MeterBundle},
    AppState,
};

use super::CameraMarker;

/// Running or paused
#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(AppState = AppState::Live)]
enum LiveState {
    /// Running
    #[default]
    Running,
    /// On pause screen
    Paused,
    /// Showing an interlude message
    ShowingInterlude,
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
            // live game take-down
            .add_systems(
                OnExit(AppState::Live),
                (despawn_all_at::<OnLive>, reset_game),
            )
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
                    mob::destroy_spawner_when_done,
                    weapon::update_cooldown,
                    weapon::trigger_weapon,
                    weapon::process_new_weapon,
                    weapon::process_approach_weapon_cube,
                    weapon::weapon_keyboard_input,
                    weapon::weapon_button_action,
                    weapon::process_weapon_change,
                    weapon::process_weapon_button_selected,
                    weapon::process_weapon_button_deselected,
                    button_system::<weapon::WeaponButton>,
                    (
                        process_player_movement,
                        apply_velocity,
                        apply_rotation,
                        stay_on_floor,
                        projectile::projectile_collision,
                    )
                        .chain(),
                    (
                        process_attacks,
                        process_target_destroyed,
                        process_new_target,
                        icon::clear_icons_of_destroyed_things,
                        process_damage_player,
                        apply_collapse,
                        time_to_live,
                        process_end_of_corridor,
                        process_live_time,
                        mob::process_spawner_trigger,
                        interlude::process_interlude_trigger,
                        button_system::<Decision>,
                        decision_action,
                    )
                        .chain(),
                )
                    .run_if(in_state(LiveState::Running)),
            )
            .add_systems(
                Update,
                (button_system::<PauseButton>, paused_button_action)
                    .run_if(in_state(LiveState::Paused)),
            )
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
            .add_event::<AdvanceInterlude>();
    }
}

fn start_running(mut next_state: ResMut<NextState<LiveState>>) {
    next_state.set(LiveState::Running);
}

fn reset_game(mut next_state: ResMut<NextState<LiveState>>, mut live_time: ResMut<LiveTime>) {
    next_state.set(LiveState::default());
    live_time.reset();
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
            LiveState::ShowingInterlude => {
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

/// Set up the main UI components in the game for the first time
fn setup_ui(mut cmd: Commands) {
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
        spawn_button_in_group(cmd, "Resume", PauseButton, PausedButtonAction::Resume);

        // button to return to main menu
        spawn_button_in_group(cmd, "Give Up", PauseButton, PausedButtonAction::GiveUp);
    });
}

fn paused_button_action(
    mut interaction_query: Query<
        (&Interaction, &PausedButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut paused_node_q: Query<&mut Style, With<PausedDiv>>,
    mut live_state: ResMut<NextState<LiveState>>,
    mut game_state: ResMut<NextState<AppState>>,
) {
    for (interaction, pause_button_action) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
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
    target_q: Query<(Entity, &Target), Without<Collapsing>>,
    mut player_q: Query<&mut PlayerMovement, With<Player>>,
) {
    let mut done = false;
    for _ in target_destroyed_events.read() {
        if done {
            // if done, we can consume the rest of the events and continue normally
            continue;
        }
        // count the number of targets still on scene
        let num_targets = target_q.iter().count();
        if num_targets == 0 {
            // let's move!
            let mut player_movement = player_q.single_mut();
            *player_movement = PlayerMovement::Walking;
            done = true;
        }
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
    if player_pos.z + 14. >= fork_transform.translation.z {
        // stop walking
        *player_movement = PlayerMovement::Idle;

        // heal player
        health.replenish();

        // and spawn new input arrows to select which way to go
        spawn_decision_arrows(&mut cmd);
    }
}

/// Marker component for the UI node containing the decision arrows
#[derive(Debug, Component)]
struct DecisionArrowsDiv;

#[derive(Debug, Component)]
enum Decision {
    Left,
    Right,
}

fn spawn_decision_arrows(cmd: &mut Commands) {
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
    mut interaction_query: Query<(&Interaction, &Decision), (Changed<Interaction>, With<Button>)>,
    mut live_state: ResMut<NextState<LiveState>>,
    mut game_state: ResMut<NextState<AppState>>,
) {
    for (interaction, decision) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // TODO
            println!("TODO apply {decision:?}");
        }
    }
}
