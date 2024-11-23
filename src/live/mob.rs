//! The mob rules

use bevy::prelude::*;
use bevy_mod_picking::{prelude::Pickable, PickableBundle};

use crate::{
    effect::ScalesUp,
    logic::{Num, TargetRule},
};

use super::{
    collision::Collidable,
    icon::{spawn_target_icon, HasIcon},
    phase::PhaseTrigger,
    player::Player,
    Health, LiveTime, OnLive, Target,
};

/// Component representing a spawner of mobs.
#[derive(Debug, Component)]
pub struct MobSpawner {
    /// time to wait between each spawn
    pub spawn_interval: f32,
    /// live time in seconds of the last spawn
    pub last_spawn: f32,
    /// the options for the target number
    pub target_options: Vec<Num>,
    pub target_rule: TargetRule,
    /// Whether it is actively spawning mobs.
    ///
    /// It starts disabled so it can be spawned at level start.
    pub active: bool,
    /// count for the number of mobs yet to be spawned
    /// (should despawn itself when it reaches 0)
    pub count: u32,
}

impl MobSpawner {
    pub fn new(count: u32, spawn_interval: f32, target_options: Vec<Num>) -> Self {
        MobSpawner::new_with_target_rule(
            count,
            spawn_interval,
            target_options,
            TargetRule::default(),
        )
    }

    pub fn new_with_target_rule(
        count: u32,
        spawn_interval: f32,
        target_options: Vec<Num>,
        target_rule: TargetRule,
    ) -> Self {
        MobSpawner {
            count,
            spawn_interval,
            target_options,
            target_rule,
            active: false,
            last_spawn: 0.,
        }
    }
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
    mut cmd: Commands,
    time: Res<LiveTime>,
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
            spawner.last_spawn = time - spawner.spawn_interval;

            // remove phase trigger
            cmd.entity(entity).remove::<PhaseTrigger>();
        }
    }
}

/// system that makes active mob spawners spawn mobs
pub fn spawn_mobs(
    mut cmd: Commands,
    time: Res<LiveTime>,
    mob_assets: Res<MobAssets>,
    mut mob_spawner_q: Query<(&mut MobSpawner, &Transform)>,
) {
    let time = time.elapsed_seconds();
    for (mut spawner, transform) in &mut mob_spawner_q {
        if !spawner.active {
            continue;
        }
        let relative_elapsed = time - spawner.last_spawn;
        if relative_elapsed >= spawner.spawn_interval {
            // spawn a mob
            // TODO use an RNG to pseudorandomize the position
            let new_pos = transform.translation + Vec3::new(0., 0., spawner.count as f32 * 0.2);
            // TODO use RNG to randomize num choice
            let new_num = spawner.target_options[0];

            spawn_mob(
                &mut cmd,
                &mob_assets,
                new_pos,
                Target {
                    num: new_num,
                    rule: spawner.target_rule,
                },
            );

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
    #[bundle()]
    pub pbr: PbrBundle,
    pub mob: Mob,
    pub collidable: Collidable,
    pub target: Target,
    pub health: Health,
    pub pickable: PickableBundle,
    pub scales_up: ScalesUp,
    pub on_live: OnLive,
}

#[derive(Debug, Resource)]
pub struct MobAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

const TARGET_SIZE: f32 = 3.;

impl FromWorld for MobAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let mesh = meshes.add(Mesh::from(Cylinder::new(TARGET_SIZE / 2., 0.25)));

        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();

        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.75, 0.25, 0.5),
            ..Default::default()
        });

        Self { mesh, material }
    }
}

pub fn spawn_mob(cmd: &mut Commands, assets: &MobAssets, position: Vec3, target: Target) {
    let num = target.num;
    let target_entity = cmd
        .spawn(MobBundle {
            pbr: PbrBundle {
                mesh: assets.mesh.clone(),
                transform: Transform {
                    // face the cylinder towards the Z axis
                    rotation: Quat::from_rotation_x(std::f32::consts::PI / 2.),
                    translation: position,
                    // start small and let it scale up
                    scale: Vec3::splat(1e-3),
                },
                material: assets.material.clone(),
                ..default()
            },
            mob: Mob,
            collidable: Collidable::new(Vec3::new(TARGET_SIZE - 0.2, TARGET_SIZE - 0.2, 0.5)),
            target,
            health: Health { value: 1., max: 1. },
            pickable: PickableBundle {
                pickable: Pickable {
                    should_block_lower: true,
                    is_hoverable: false,
                },
                ..Default::default()
            },
            scales_up: ScalesUp,
            on_live: OnLive,
        })
        .id();

    // spawn icon
    let icon_entity = spawn_target_icon(cmd, target_entity, num);

    // add reverse reference
    cmd.entity(target_entity).insert(HasIcon(icon_entity));
}
