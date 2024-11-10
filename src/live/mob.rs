//! The mob rules

use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;

use super::{Health, Target};

/// Component representing a spawner of mobs.
#[derive(Debug, Component)]
pub struct MobSpawner {
    /// count for the number of mobs yet to be spawned
    /// (should despawn itself when it reaches 0)
    pub count: u32,
    pub spawn_interval: f32,
    pub last_spawn: f32,
}

pub fn destroy_spawner_when_done(mut q: Query<(Entity, &MobSpawner)>, mut commands: Commands) {
    for (entity, spawner) in q.iter_mut() {
        if spawner.count == 0 {
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
