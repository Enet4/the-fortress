//! Components, systems, and other functions specific to the player

use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    assets::AudioHandles,
    cheat::Cheats,
    effect::{Collapsing, StaysOnFloor, TimeToLive, Velocity},
    live::Target,
    logic::{test_attack_on, AttackTest},
    postprocess::PostProcessSettings,
    ui::{set_meter_value, Meter},
};

use super::{
    weapon::{AttackCooldown, PlayerAttack},
    CooldownMeter, Health, HealthMeter, LiveState, OnLive,
};

/// Marker for the player
#[derive(Debug, Default, Component)]
pub struct Player;

#[derive(Debug, Default, Bundle)]
pub struct PlayerBundle {
    player: Player,
    player_movement: PlayerMovement,
    velocity: Velocity,
    health: Health,
    attack_cooldown: AttackCooldown,
    #[bundle()]
    transform: TransformBundle,
    #[bundle()]
    visibility: VisibilityBundle,
    on_live: OnLive,
}

/// The state of the player in terms of movement
#[derive(Debug, Default, Component)]
pub enum PlayerMovement {
    /// Idle, usually awaiting input or facing enemies
    Idle,
    /// Moving along the corridor
    #[default]
    Walking,
    /// Moving slowly along the corridor, usually temporary
    Slower,
    /// Stopping abruptly
    Halting,
}

pub fn process_player_movement(
    time: Res<Time>,
    mut query: Query<(&PlayerMovement, &mut Velocity), With<Player>>,
) {
    let elapsed = time.delta_seconds();

    const MAX_SPEED: f32 = 11.;
    const SLOW_SPEED: f32 = 5.;

    for (movement, mut velocity) in query.iter_mut() {
        match movement {
            PlayerMovement::Idle => {
                // slowly decrease Z velocity
                velocity.0.z = (velocity.0.z * 0.78 / (1. + elapsed)).max(0.);
            }
            PlayerMovement::Walking => {
                // increase Z velocity up to a maximum
                velocity.0.z = (velocity.0.z + 8. * elapsed).min(MAX_SPEED);
            }
            PlayerMovement::Slower => {
                // adjust Z velocity until it reaches the one desired
                if velocity.0.z > SLOW_SPEED {
                    velocity.0.z = (velocity.0.z - 6. * elapsed).max(SLOW_SPEED);
                } else {
                    velocity.0.z = (velocity.0.z + 6. * elapsed).min(SLOW_SPEED);
                }
            }
            PlayerMovement::Halting => {
                // stop the player
                velocity.0.z = 0.;
            }
        }
    }
}

/// create and spawn a new player entity
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
        ..default()
    })
}

#[derive(Debug, Event)]
pub struct TargetDestroyed;

/// system for processing player attacks
pub fn process_attacks(
    mut cmd: Commands,
    audio_sources: Res<AudioHandles>,
    mut events: EventReader<PlayerAttack>,
    mut damage_player_events: EventWriter<DamagePlayer>,
    mut target_destroyed_events: EventWriter<TargetDestroyed>,
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

        // apply the attack
        match attack_result {
            AttackTest::Effective(new_num) => {
                if let Some(mut health) = health {
                    // damage the target
                    health.value -= 1.;
                    if health.value <= 0. {
                        // add the effects to destroy the target
                        cmd.entity(*entity).remove::<Target>().insert((
                            Collapsing::default(),
                            Velocity(Vec3::new(0., 8., 6.)),
                            TimeToLive(0.5),
                        ));

                        audio_sources.play_hit02(&mut cmd);

                        target_destroyed_events.send(TargetDestroyed);
                    } else {
                        // update target with its new number
                        if let Some(num) = new_num {
                            target.num = num;
                        }
                    }
                } else {
                    // with no health, the target is destroyed
                    cmd.entity(*entity).remove::<Target>().insert((
                        Collapsing::default(),
                        Velocity(Vec3::new(0., 8., 6.)),
                        TimeToLive(0.5),
                    ));

                    // send event for target destroyed
                    target_destroyed_events.send(TargetDestroyed);
                }
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
    audio_sources: Res<AudioHandles>,
    cheats: Res<Cheats>,
    mut player_q: Query<(Entity, &mut Health), With<Player>>,
    mut postprocess_settings_q: Query<&mut PostProcessSettings>,
    mut next_state: ResMut<NextState<LiveState>>,
) {
    if cheats.invulnerability {
        return;
    }

    for DamagePlayer { damage } in events.read() {
        audio_sources.play_hit37(&mut cmd);

        let Ok((player_entity, mut player_health)) = player_q.get_single_mut() else {
            return;
        };
        player_health.value -= damage;

        // update postprocess settings
        if let Ok(mut settings) = postprocess_settings_q.get_single_mut() {
            settings.add_intensity(0.5);
            if player_health.value <= 0.2 {
                settings.oscillate = 0.48;
            } else if player_health.value < 0.3 {
                settings.oscillate = 0.25;
            } else if player_health.value < 0.5 {
                settings.oscillate = 0.1;
            } else {
                settings.oscillate = 0.;
            }
        }

        if player_health.value <= 0. {
            // player is dead
            cmd.entity(player_entity).insert((
                Velocity(Vec3::new(0., 4., 3.)),
                Collapsing::default(),
                StaysOnFloor,
            ));
            // enter try again screen
            next_state.set(LiveState::Defeat)
        }
    }
}

/// system for updating the cooldown meter
/// based on the selected weapon cooldown
pub fn update_player_cooldown_meter(
    query: Query<&AttackCooldown, With<Player>>,
    mut meter_query: Query<(&mut Style, &mut BackgroundColor), (With<Meter>, With<CooldownMeter>)>,
) {
    // we only expect 1 selected weapon
    let Ok(cooldown) = query.get_single() else {
        return;
    };
    let percent = 100. * cooldown.value / cooldown.max;

    for (mut style, mut background_color) in meter_query.iter_mut() {
        style.width = Val::Percent(percent);
        if cooldown.locked {
            background_color.0 = Color::WHITE;
        } else {
            background_color.0 = Color::srgba_u8(0, 63, 255, 224);
        }
    }
}

/// system for updating the player's health meter
pub fn update_player_health_meter(
    query: Query<&Health, With<Player>>,
    meter_query: Query<&mut Style, (With<Meter>, With<HealthMeter>)>,
) {
    // we only expect 1 selected weapon
    let Ok(health) = query.get_single() else {
        return;
    };
    let percent = 100. * health.value / health.max;
    set_meter_value(meter_query, percent);
}
