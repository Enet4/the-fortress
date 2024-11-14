//! The live action module, containing the active game logic
use bevy::{prelude::*, ui::FocusPolicy};
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::{Listener, PointerButton},
};

pub mod collision;
mod mob;
pub mod obstacle;
mod player;
mod projectile;
mod scene;
mod weapon;

use bevy_ui_anchor::{AnchorTarget, AnchorUiNode, HorizontalAnchor, VerticalAnchor};
use player::{
    process_attacks, process_damage_player, process_player_movement, update_player_cooldown_meter,
    update_player_health_meter, DamagePlayer, Player, PlayerMovement, TargetDestroyed,
};
use projectile::ProjectileAssets;
use weapon::{install_weapon, PlayerAttack};
// re-export some stuff
pub use scene::setup_scene;
pub use weapon::TriggerWeapon;

use crate::{
    effect::{
        apply_collapse, apply_torque, apply_velocity, stay_on_floor, time_to_live, Collapsing,
    },
    logic::{Num, TargetRule},
    structure::Fork,
    ui::MeterBundle,
};

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

/// Marker component for the UI node showing the number of the target
#[derive(Debug, Component)]
pub struct TargetIconNode;

/// system to despawn target icon nodes when
/// the target that they are representing is destroyed
pub fn clear_collapsed_target_icons(
    mut cmd: Commands,
    collapsed_targets_q: Query<Entity, With<Collapsing>>,
    target_icon_q: Query<(Entity, &AnchorUiNode), With<TargetIconNode>>,
) {
    for entity in collapsed_targets_q.iter() {
        for (icon_entity, anchor) in target_icon_q.iter() {
            let anchor_target = &anchor.target;
            if matches!(anchor_target, AnchorTarget::Entity(e) if entity == *e) {
                cmd.entity(icon_entity).despawn_recursive();
            }
        }
    }
}

/// Spawn a node that shows the target number on top of the target
pub fn spawn_target_icon(cmd: &mut Commands, entity: Entity, num: Num) -> Entity {
    // draw a circle
    cmd.spawn((
        TargetIconNode,
        NodeBundle {
            style: Style {
                align_self: AlignSelf::Center,
                margin: UiRect::all(Val::Auto),
                width: Val::Px(40.),
                height: Val::Px(40.),
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0., 0., 0., 0.825)),
            border_radius: BorderRadius::all(Val::Percent(50.)),
            ..default()
        },
        AnchorUiNode {
            anchorwidth: HorizontalAnchor::Mid,
            anchorheight: VerticalAnchor::Mid,
            target: AnchorTarget::Entity(entity),
        },
    ))
    .with_children(|cmd| {
        // and draw the number in the circle
        cmd.spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::Center,
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            text: Text::from_section(
                num.to_string(),
                TextStyle {
                    font_size: 36.,
                    ..default()
                },
            ),
            ..default()
        });
    })
    .id()
}

/// Component for the player's attack cooldown meter
#[derive(Debug, Default, Component)]
pub struct CooldownMeter;

/// Component for the player's health meter
#[derive(Debug, Default, Component)]
pub struct HealthMeter;

/// Marker for the main camera
#[derive(Component)]
pub struct CameraMarker;

/// The plugin which adds everything related to the live action
pub struct LiveActionPlugin;

impl Plugin for LiveActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_ui_anchor::AnchorUiPlugin::<CameraMarker>::new())
            .add_systems(Startup, (setup_ui, install_first_weapon))
            .add_systems(
                Update,
                (
                    update_player_cooldown_meter,
                    update_player_health_meter,
                    mob::destroy_spawner_when_done,
                    weapon::update_cooldown,
                    weapon::trigger_weapon,
                    weapon::process_new_weapon,
                    (
                        process_player_movement,
                        apply_velocity,
                        apply_torque,
                        stay_on_floor,
                        projectile::projectile_collision,
                    )
                        .chain(),
                    (
                        process_attacks,
                        process_target_destroyed,
                        clear_collapsed_target_icons,
                        process_damage_player,
                        apply_collapse,
                        time_to_live,
                        process_end_of_corridor,
                    )
                        .chain(),
                ),
            )
            // resources
            .init_resource::<ProjectileAssets>()
            // events
            .add_event::<TriggerWeapon>()
            .add_event::<PlayerAttack>()
            .add_event::<TargetDestroyed>()
            .add_event::<DamagePlayer>();
    }
}

fn install_first_weapon(cmd: Commands) {
    install_weapon(cmd, 3.into());
}

/// Marker component for a UI node containing the weapon selectors
#[derive(Debug, Default, Component)]
pub struct WeaponListNode;

/// Marker component for a weapon button
#[derive(Debug, Default, Component)]
pub struct WeaponButton;

fn setup_ui(mut cmd: Commands) {
    // Node that fills entire background
    cmd.spawn(NodeBundle {
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
    })
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
}

/// create a new button
pub fn spawn_weapon_button(cmd: &mut ChildBuilder<'_>, attack_num: Num, shortcut: char) {
    // insert button
    cmd.spawn((
        WeaponButton,
        ButtonBundle {
            background_color: BackgroundColor(Color::BLACK),
            border_color: BorderColor(Color::WHITE),
            border_radius: BorderRadius::all(Val::Px(1.)),
            style: Style {
                border: UiRect::all(Val::Px(1.)),
                display: Display::Flex,
                align_self: AlignSelf::Center,
                column_gap: Val::Px(10.),
                width: Val::Px(64.),
                height: Val::Px(64.),
                margin: UiRect::all(Val::Px(10.)),
                ..default()
            },
            ..default()
        },
    ))
    .with_children(|parent| {
        // shortcut
        parent.spawn(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(1.),
                top: Val::Px(1.),
                ..default()
            },
            text: Text::from_section(
                shortcut.to_string(),
                TextStyle {
                    font_size: 14.,
                    ..default()
                },
            ),
            ..Default::default()
        });

        // the actual number of the attack
        parent.spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::Center,
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            text: Text::from_section(
                attack_num.to_string(),
                TextStyle {
                    font_size: 36.,
                    ..default()
                },
            ),
            ..default()
        });
    });
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
    }
}
