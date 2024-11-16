//! The mob rules

use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;

use super::{phase::PhaseTrigger, player::Player, Health, LiveTime, Target};

/// Component representing a spawner of mobs.
#[derive(Debug, Component)]
pub struct MobSpawner {
    /// whether it has been triggered
    pub active: bool,
    /// count for the number of mobs yet to be spawned
    /// (should despawn itself when it reaches 0)
    pub count: u32,
    pub spawn_interval: f32,
    /// live time in seconds of the last spawn
    pub last_spawn: f32,
}

pub fn destroy_spawner_when_done(mut q: Query<(Entity, &MobSpawner)>, mut commands: Commands) {
    for (entity, spawner) in q.iter_mut() {
        if spawner.count == 0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn process_spawner_trigger(
    time: Res<LiveTime>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut MobSpawner, &PhaseTrigger)>,
    player_q: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_q.get_single() else {
        return;
    };

    let time = time.elapsed_seconds();
    for (entity, mut spawner, phase) in q.iter_mut() {
        if phase.should_trigger(&player_transform.translation) {
            spawner.active = true;
            spawner.last_spawn = time;
        }
        // despawn trigger
        if spawner.active && spawner.count == 0 {
            commands.entity(entity).despawn();
        }
    }
}

/// The enemies that appear.
#[derive(Debug, Default, Component)]
pub struct Mob;

#[derive(Default, Bundle)]
pub struct MobBundle {
    pub mesh: Handle<Mesh>,
    pub mob: Mob,
    pub target: Target,
    pub health: Health,
    pub pickable: PickableBundle,
}
