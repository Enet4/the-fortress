use bevy::{
    math::bounding::{BoundingSphere, IntersectsVolume as _},
    prelude::*,
};

use crate::{effect::Velocity, logic::Num};

use super::{
    collision::CollidableBox,
    weapon::{PlayerAttack, PlayerWeapon},
    OnLive, Target,
};

#[derive(Debug, Clone, Resource)]
pub struct ProjectileAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

impl FromWorld for ProjectileAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh: Handle<Mesh> = meshes.add(Sphere::new(0.12));

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material = materials.add(StandardMaterial {
            emissive: LinearRgba::new(1., 0.825, 0.5, 0.75),
            emissive_exposure_weight: 0.0,
            ..Default::default()
        });

        ProjectileAssets { mesh, material }
    }
}

/// Marker for a projectile
#[derive(Debug, Default, Component)]
pub struct Projectile {
    /// the number which defines the kind of attack
    pub num: Num,
}

/// Bundle for a projectile
#[derive(Debug, Default, Bundle)]
pub struct ProjectileBundle {
    pub projectile: Projectile,
    pub velocity: Velocity,
    #[bundle()]
    pub transform: TransformBundle,
}

pub fn spawn_projectile(
    cmd: &mut Commands,
    player_position: Vec3,
    direction: Vec3,
    weapon: &PlayerWeapon,
    assets: &ProjectileAssets,
) {
    // spawn a projectile
    let pos = player_position + Vec3::new(0.15, 0.25, 1.);

    cmd.spawn((
        OnLive,
        Projectile { num: weapon.num },
        PbrBundle {
            visibility: Visibility::Visible,
            transform: Transform::from_translation(pos),
            mesh: assets.mesh.clone(),
            material: assets.material.clone(),
            ..default()
        },
        Velocity(direction * weapon.projectile_speed),
    ))
    .with_children(|cmd| {
        // add a light to the projectile
        cmd.spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::srgb(1., 0.825, 0.5),
                intensity: 4_400.0,
                range: 14.0,
                ..Default::default()
            },
            ..default()
        });
    });
}

/// System for handling the collision of projectiles
pub fn projectile_collision(
    mut cmd: Commands,
    projectile_q: Query<(Entity, &Transform, &Projectile)>,
    collidable_q: Query<(Entity, &CollidableBox, &Transform, Option<&Target>)>,
    mut attack_events: EventWriter<PlayerAttack>,
) {
    for (p_entity, p_transform, projectile) in projectile_q.iter() {
        for (entity, collidable, t_transform, target) in collidable_q.iter() {
            let bound = collidable.to_bound(t_transform.translation);
            if bound.intersects(&BoundingSphere::new(p_transform.translation, 0.25)) {
                if target.is_some() {
                    // send event
                    attack_events.send(PlayerAttack {
                        entity,
                        num: projectile.num,
                    });
                }
                // despawn the projectile (and respective light)
                // TODO particles
                cmd.entity(p_entity).despawn_recursive();

                // should not hit any other target
                break;
            }
        }
    }
}
