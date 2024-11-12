use bevy::{
    math::bounding::{BoundingSphere, IntersectsVolume as _},
    prelude::*,
};

use crate::{effect::Velocity, logic::Num};

use super::{collision::Collidable, weapon::PlayerAttack, Target};

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
                if target.is_some() {
                    // send event
                    attack_events.send(PlayerAttack {
                        entity,
                        num: projectile.num,
                    });
                }
                // despawn the projectile
                // TODO particles
                cmd.entity(p_entity).despawn();
            }
        }
    }
}
