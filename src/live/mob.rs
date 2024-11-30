//! The mob rules

use bevy::prelude::*;
use bevy_mod_picking::{prelude::Pickable, PickableBundle};
use tinyrand::RandRange;

use crate::{
    effect::ScalesUp,
    logic::{Num, TargetRule},
};

use super::{
    collision::CollidableBox,
    icon::{spawn_target_icon, HasIcon},
    phase::PhaseTrigger,
    player::{Player, TargetDestroyed},
    Health, LiveTime, OnLive, Target,
};

/// Component representing a spawner of mobs.
#[derive(Debug, Clone, Component)]
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

/// Component for things containing some form of randomness.
/// Contains a random number generator.
#[derive(Component)]
pub struct Randomness {
    pub rng: tinyrand::SplitMix,
}

impl MobSpawner {
    pub fn new<I>(count: u32, spawn_interval: f32, target_options: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Num>,
    {
        MobSpawner::new_with_target_rule(
            count,
            spawn_interval,
            target_options,
            TargetRule::default(),
        )
    }

    pub fn new_with_target_rule<I>(
        count: u32,
        spawn_interval: f32,
        target_options: I,
        target_rule: TargetRule,
    ) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Num>,
    {
        MobSpawner {
            count,
            spawn_interval,
            target_options: target_options.into_iter().map(Into::into).collect(),
            target_rule,
            active: false,
            last_spawn: 0.,
        }
    }
}

#[derive(Bundle)]
pub struct MobSpawnerBundle {
    pub phase_trigger: PhaseTrigger,
    pub transform: Transform,
    pub spawner: MobSpawner,
    pub random: Randomness,
    pub on_live: OnLive,
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

/// Z offset where mobs are spawned
/// relative to the mob spawner position
const MOB_SPAWN_Z_OFFSET: f32 = 12.;

/// system that makes active mob spawners spawn mobs
pub fn spawn_mobs_on_time(
    mut cmd: Commands,
    time: Res<LiveTime>,
    mob_assets: Res<MobAssets>,
    mut mob_spawner_q: Query<(&mut MobSpawner, &mut Randomness, &Transform)>,
) {
    let time = time.elapsed_seconds();
    for (mut spawner, mut random, transform) in &mut mob_spawner_q {
        if !spawner.active {
            continue;
        }
        let relative_elapsed = time - spawner.last_spawn;
        if relative_elapsed >= spawner.spawn_interval {
            // spawn a mob
            // use an RNG to pseudorandomize the position
            let rel_x = (random.rng.next_range(0..14_u32) as f32 - 7.) / 2.;
            let rel_y = random.rng.next_range(0..5_u32) as f32 - 2.5;
            let rel_z = if spawner.count % 2 == 0 {
                MOB_SPAWN_Z_OFFSET + (spawner.count / 2) as f32 * 0.2
            } else {
                MOB_SPAWN_Z_OFFSET - (spawner.count / 2) as f32 * 0.2
            };
            let new_pos = transform.translation + Vec3::new(rel_x, rel_y, rel_z);

            let choice = random
                .rng
                .next_range(0..spawner.target_options.len() as u32);
            // randomize num choice
            let new_num = spawner.target_options[choice as usize];

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

/// system that makes mob spawners spawn immediately when there are no targets left
pub fn hurry_mob_spawners_on_no_targets(
    time: Res<LiveTime>,
    mut mob_spawner_q: Query<(&mut MobSpawner, &mut Randomness, &Transform)>,
    target_q: Query<Entity, With<Target>>,
    mut events: EventReader<TargetDestroyed>,
) {
    // only act upon the target destroyed event
    if events.read().count() == 0 {
        return;
    }

    // only act if there are no targets left
    if !target_q.is_empty() {
        return;
    }

    // grab one of the mob spawners and readjust last spawn time,
    // so that a mob is spawned shortly after
    for (mut spawner, _, _) in &mut mob_spawner_q.iter_mut() {
        if spawner.active && spawner.count > 0 {
            spawner.last_spawn = time.elapsed_seconds() - spawner.spawn_interval + 0.15;
            break;
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
    pub collidable: CollidableBox,
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

const TARGET_SIZE: f32 = 2.75;

impl FromWorld for MobAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let mesh = meshes.add(Mesh::from(Cylinder::new(TARGET_SIZE / 2., 0.25)));

        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();

        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.66, 0.125, 0.5),
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
            collidable: CollidableBox::new(Vec3::new(TARGET_SIZE - 0.25, TARGET_SIZE - 0.25, 0.25)),
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
