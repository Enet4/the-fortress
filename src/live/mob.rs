//! The mob rules

use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;

use super::{phase::PhaseTrigger, player::Player, Health, LiveTime, OnLive, Target};

/// Component representing a spawner of mobs.
#[derive(Debug, Component)]
pub struct MobSpawner {
    /// Whether it is actively spawning mobs.
    ///
    /// It starts disabled so it can be spawned at level start.
    pub active: bool,
    /// count for the number of mobs yet to be spawned
    /// (should despawn itself when it reaches 0)
    pub count: u32,
    /// time to wait between each spawn
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

/// system that activates mob spawners when they approach a phase trigger
pub fn process_spawner_trigger(
    time: Res<LiveTime>,
    mut q: Query<(&mut MobSpawner, &PhaseTrigger)>,
    player_q: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_q.get_single() else {
        return;
    };

    let time = time.elapsed_seconds();
    for (mut spawner, phase) in q.iter_mut() {
        if phase.should_trigger(&player_transform.translation) {
            spawner.active = true;
            spawner.last_spawn = time;
        }
    }
}

/// system that makes active mob spawners spawn mobs
pub fn spawn_mobs(
    mut cmd: Commands,
    time: Res<LiveTime>,
    mut mob_spawner_q: Query<(&mut MobSpawner, &Transform)>,
) {
    let time = time.elapsed_seconds();
    for (mut spawner, transform) in &mut mob_spawner_q {
        if !spawner.active {
            continue;
        }

        if time - spawner.last_spawn >= spawner.spawn_interval {
            // spawn a mob
            let translation = transform.translation;

            // update spawner properties
            spawner.last_spawn += spawner.spawn_interval;
            spawner.count -= 1;
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
    pub on_live: OnLive,
}

#[derive(Debug, Resource)]
pub struct MobAssets {
    mesh: Handle<Mesh>,
}

impl FromWorld for MobAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let mesh = meshes.add(Mesh::from(Cylinder::new(5., 0.25)));
        Self { mesh }
    }
}

pub fn spawn_mob(cmd: &mut Commands, assets: Res<MobAssets>, position: Vec3, target: Target) {}
