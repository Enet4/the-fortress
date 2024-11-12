//! The live action module, containing the active game logic
use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::{Listener, PointerButton},
};

pub mod collision;
mod mob;
mod projectile;
mod weapon;

use weapon::{AttackCooldown, PlayerAttack, SelectedWeapon};
// re-export events
pub use weapon::TriggerWeapon;

use crate::{
    effect::{
        apply_collapse, apply_torque, apply_velocity, stay_on_floor, time_to_live, Collapsing,
        TimeToLive, Velocity,
    },
    logic::{test_attack_on, AttackTest, Num, TargetRule},
    postprocess::PostProcessSettings,
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

    pub fn heal(&mut self, amount: f32) {
        self.value = (self.value + amount).min(self.max);
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

/// Marker for the player
#[derive(Debug, Default, Component)]
pub struct Player;

#[derive(Debug, Default, Bundle)]
pub struct PlayerBundle {
    player: Player,
    health: Health,
    selected_weapon: SelectedWeapon,
    attack_cooldown: AttackCooldown,
    #[bundle()]
    transform: TransformBundle,
    #[bundle()]
    visibility: VisibilityBundle,
}

/// The plugin which adds everything related to the live action
pub struct LiveActionPlugin;

impl Plugin for LiveActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (
                    mob::destroy_spawner_when_done,
                    weapon::update_cooldown,
                    weapon::trigger_weapon,
                    (
                        apply_velocity,
                        apply_torque,
                        stay_on_floor,
                        projectile::projectile_collision,
                    )
                        .chain(),
                    (
                        process_attacks,
                        process_damage_player,
                        apply_collapse,
                        time_to_live,
                    )
                        .chain(),
                ),
            )
            .add_event::<TriggerWeapon>()
            .add_event::<PlayerAttack>()
            .add_event::<DamagePlayer>();
    }
}

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
        // TODO position weapon selector icons

        // insert button
        root.spawn(ButtonBundle {
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
        })
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
                    "1",
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
                    "#",
                    TextStyle {
                        font_size: 36.,
                        ..default()
                    },
                ),
                ..Default::default()
            });
        });

        // insert cooldown meter
        root.spawn(MeterBundle::new(
            Val::Px(8.),
            Color::srgba_u8(0, 63, 255, 224),
        ));

        // insert health meter
        root.spawn(MeterBundle::new(
            Val::Px(48.),
            Color::srgba_u8(0, 255, 3, 255),
        ));
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

pub fn spawn_player<'a>(cmd: &'a mut Commands, position: Vec3) -> EntityCommands<'a> {
    cmd.spawn(PlayerBundle {
        transform: TransformBundle {
            local: Transform::from_translation(position),
            ..default()
        },
        visibility: VisibilityBundle {
            visibility: Visibility::Hidden,
            inherited_visibility: InheritedVisibility::VISIBLE,
            ..default()
        },
        health: Health::new(8.),
        selected_weapon: SelectedWeapon {
            num: Num::ONE,
            ..default()
        },
        ..default()
    })
}

/// system for processing player attacks
pub fn process_attacks(
    mut cmd: Commands,
    mut events: EventReader<PlayerAttack>,
    mut damage_player_events: EventWriter<DamagePlayer>,
    mut target_query: Query<(&mut Target, Option<&mut Health>)>,
) {
    for PlayerAttack { entity, num } in events.read() {
        // query entity for target information
        let Ok((mut target, health)) = target_query.get_mut(*entity) else {
            eprintln!("no target found for attack");
            return;
        };

        // evaluate the attack
        let attack_result = test_attack_on(&target, *num);

        println!("Attack result: {:?}", attack_result);
        // apply the attack
        match attack_result {
            AttackTest::Effective(None) => {
                if let Some(mut health) = health {
                    // damage the target
                    health.value -= 1.;
                    if health.value <= 0. {
                        // add the effects to destroy the target
                        cmd.entity(*entity).remove::<Target>().insert((
                            Collapsing::default(),
                            Velocity(Vec3::new(0., 12., 6.)),
                            TimeToLive(0.75),
                        ));
                    }
                } else {
                    // with no health, the target is destroyed
                    cmd.entity(*entity).remove::<Target>().insert((
                        Collapsing::default(),
                        Velocity(Vec3::new(0., 12., 6.)),
                        TimeToLive(0.75),
                    ));
                }
            }
            AttackTest::Effective(Some(new_num)) => {
                target.num = new_num;
            }
            AttackTest::Failed => {
                // nope, damage the player back
                damage_player_events.send(DamagePlayer { damage: 1. });
            }
        }
    }
}

#[derive(Debug, Event)]
pub struct DamagePlayer {
    pub damage: f32,
}

pub fn process_damage_player(
    mut cmd: Commands,
    mut events: EventReader<DamagePlayer>,
    mut player_q: Query<(Entity, &mut Health), With<Player>>,
    mut postprocess_settings_q: Query<&mut PostProcessSettings>,
) {
    for DamagePlayer { damage } in events.read() {
        // TODO play sound effect

        let Ok((player_entity, mut player_health)) = player_q.get_single_mut() else {
            return;
        };
        player_health.value -= damage;

        // update postprocess settings
        for mut settings in postprocess_settings_q.iter_mut() {
            settings.intensity = (settings.intensity + 0.5).min(0.75);
            if player_health.value < 0.125 {
                settings.oscillate = 0.45;
            } else if player_health.value < 0.25 {
                settings.oscillate = 0.25;
            } else if player_health.value < 0.5 {
                settings.oscillate = 0.1;
            } else {
                settings.oscillate = 0.01;
            }
        }

        if player_health.value <= 0. {
            // player is dead
            cmd.entity(player_entity).insert(Collapsing::default());
        }
    }
}
