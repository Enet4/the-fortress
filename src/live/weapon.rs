use bevy::prelude::*;

use crate::{
    live::{spawn_weapon_button, weapon},
    logic::Num,
};

use super::{
    player::Player,
    projectile::{spawn_projectile, ProjectileAssets},
    WeaponListNode,
};

/// Component representing a specific weapon in the player's arsenal.
/// (in practice it is always the staff, but the projectiles are different)
#[derive(Debug, Component)]
pub struct Weapon {
    /// projectile speed
    pub projectile_speed: f32,
    /// the number representing the attack
    pub num: Num,
    /// the amount of cooldown added per use
    pub cooldown: f32,
}

impl Weapon {
    pub fn new(num: Num) -> Self {
        Self {
            num,
            ..Default::default()
        }
    }
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            projectile_speed: 25.,
            num: 0.into(),
            cooldown: 1.,
        }
    }
}

pub fn install_weapon(mut cmd: Commands, num: Num) {
    cmd.spawn(Weapon::new(num));
}

/// Marker component representing the weapon currently wielded by the player.
#[derive(Debug, Default, Component)]
pub struct WeaponSelected;

/// system that processes the addition of new weapons
pub fn process_new_weapon(
    mut cmd: Commands,
    weapon_q: Query<(Entity, &Weapon), Added<Weapon>>,
    mut weapon_list_node_q: Query<(Entity, Option<&Children>), With<WeaponListNode>>,
) {
    for (weapon_entity, weapon) in weapon_q.iter() {
        println!("New weapon added {weapon:?}");

        // add a new weapon to the list
        let (entity, weapon_buttons) = weapon_list_node_q
            .get_single_mut()
            .expect("No weapon list node found! This is likely a bug");

        let shortcut = if let Some(weapon_buttons) = weapon_buttons {
            if weapon_buttons.is_empty() {
                // automatically select the first weapon
                cmd.entity(weapon_entity).insert(WeaponSelected);
            }

            weapon_buttons.len() as u8 + 1
        } else {
            // automatically select the first weapon
            cmd.entity(weapon_entity).insert(WeaponSelected);
            1
        };
        cmd.entity(entity).with_children(|root| {
            spawn_weapon_button(root, weapon.num, (shortcut + b'0') as char);
        });
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
    projectile_assets: Res<ProjectileAssets>,
    mut trigger_weapon_events: EventReader<TriggerWeapon>,
    mut weapon_q: Query<&Weapon, With<WeaponSelected>>,
    mut player_q: Query<(&GlobalTransform, &mut AttackCooldown), With<Player>>,
) {
    for trigger_weapon in trigger_weapon_events.read() {
        let Ok(weapon) = weapon_q.get_single_mut() else {
            return;
        };

        let (player_transform, mut cooldown) = player_q.single_mut();

        // if the weapon is locked, we cannot trigger it
        if cooldown.locked {
            continue;
        }

        let player_position = player_transform.translation();

        // TODO play sound effect

        let direction = trigger_weapon.target_pos - player_position;
        let direction = direction.normalize();

        // spawn a projectile
        spawn_projectile(
            &mut cmd,
            player_position,
            direction,
            weapon,
            &projectile_assets,
        );

        // apply cooldown
        cooldown.value = cooldown.value + weapon.cooldown;
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
