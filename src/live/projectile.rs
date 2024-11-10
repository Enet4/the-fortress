use bevy::{
    math::bounding::{BoundingSphere, IntersectsVolume as _},
    prelude::*,
};

use crate::logic::Num;

use super::{collision::Collidable, weapon::PlayerAttack, Target};

/// Component for things which fly at a fixed speed
#[derive(Debug, Default, Component)]
pub struct Velocity(pub Vec3);

pub fn apply_velocity(time: Res<Time>, mut q: Query<(&mut Transform, &Velocity)>) {
    let delta = time.delta_seconds();
    for (mut transform, velocity) in q.iter_mut() {
        transform.translation += Vec3::new(velocity.0.x, velocity.0.y, velocity.0.z) * delta;
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

/// System for handling the collision of projectiles
pub fn projectile_collision(
    mut cmd: Commands,
    projectile_q: Query<(Entity, &Transform, &Projectile)>,
    collidable_q: Query<(Entity, &Collidable, &Transform, Option<&Target>)>,
    mut attack_events: EventWriter<PlayerAttack>,
) {
    for (p_entity, p_transform, projectile) in projectile_q.iter() {
        for (entity, collidable, t_transform, target) in collidable_q.iter() {
            let bound = collidable.to_bound(t_transform.translation);
            if bound.intersects(&BoundingSphere::new(p_transform.translation, 0.5)) {
                if let Some(target) = target {
                    println!("hit a target: {:?}", target);
                    // send event
                    attack_events.send(PlayerAttack {
                        entity,
                        num: projectile.num,
                    });
                }
                println!("hit entity {:?}", entity);
                // despawn the projectile
                cmd.entity(p_entity).despawn();
            }
        }
    }
}
