use bevy::prelude::*;

use crate::{effect::Velocity, logic::Num};

use super::{projectile::Projectile, Player};

/// Component representing weapon currently wielded by the player
#[derive(Debug, Component)]
pub struct SelectedWeapon {
    /// projectile speed
    pub projectile_speed: f32,
    /// the number representing the attack
    pub num: Num,
    /// the amount of cooldown added per use
    pub cooldown: f32,
}

impl Default for SelectedWeapon {
    fn default() -> Self {
        Self {
            projectile_speed: 24.,
            num: 0.into(),
            cooldown: 1.,
        }
    }
}

/// Component for implementing a timeout before
/// the next attack can be made by a player or mob.
#[derive(Debug, Component)]
pub struct AttackCooldown {
    /// the time to wait before the next attack, in seconds
    pub value: f32,
    /// the maximum cooldown, usually applied after an attack, in seconds
    pub max: f32,
    /// whether the weapon cannot be used (because it overheated)
    pub locked: bool,
}

impl Default for AttackCooldown {
    fn default() -> Self {
        Self {
            value: 0.,
            max: 2.,
            locked: false,
        }
    }
}

pub fn update_cooldown(time: Res<Time>, mut q: Query<&mut AttackCooldown>) {
    for mut cooldown in q.iter_mut() {
        cooldown.value -= time.delta_seconds();
        if cooldown.value <= 0. {
            cooldown.value = 0.;
            cooldown.locked = false;
        }
    }
}

/// An event fired when the player clicks on something to attack.
#[derive(Debug, Event)]
pub struct TriggerWeapon {
    pub target_pos: Vec3,
}

/// System that reacts to events for triggering the weapon.
pub fn trigger_weapon(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut trigger_weapon_events: EventReader<TriggerWeapon>,
    mut player_q: Query<(&GlobalTransform, &SelectedWeapon, &mut AttackCooldown), With<Player>>,
) {
    for trigger_weapon in trigger_weapon_events.read() {
        let (player_transform, selected_weapon, mut cooldown) = player_q.single_mut();

        // if the weapon is locked, we cannot trigger it
        if cooldown.locked {
            continue;
        }

        let player_position = player_transform.translation();

        // TODO play sound effect

        let diff = trigger_weapon.target_pos - player_position;
        let diff = diff.normalize();

        // spawn a projectile
        let pos = player_position + Vec3::new(0.15, 0.25, 1.);

        cmd.spawn((
            Projectile {
                num: selected_weapon.num,
            },
            PbrBundle {
                visibility: Visibility::Visible,
                transform: Transform::from_translation(pos),
                mesh: meshes.add(Sphere::new(0.12)).into(),
                material: materials.add(StandardMaterial {
                    emissive: LinearRgba::new(1., 0.825, 0.5, 0.75),
                    emissive_exposure_weight: 0.0,
                    ..Default::default()
                }),
                ..default()
            },
            Velocity(diff * selected_weapon.projectile_speed),
        ));

        // apply cooldown
        cooldown.value = cooldown.value + selected_weapon.cooldown;
        if cooldown.value >= cooldown.max {
            cooldown.value = cooldown.max;
            cooldown.locked = true;
        }
    }
}

/// An event fired when a player projectile hits a target.
#[derive(Debug, Event)]
pub struct PlayerAttack {
    /// the target entity hit by the attack
    pub entity: Entity,
    /// the number of the attack
    pub num: Num,
}
