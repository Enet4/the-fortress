//! Components and systems for miscellaneous effects
use bevy::prelude::*;

use crate::live::LiveTime;

/// Component for things which fly at a fixed speed
#[derive(Debug, Default, Component)]
pub struct Velocity(pub Vec3);

pub fn apply_velocity(time: Res<Time>, mut q: Query<(&mut Transform, &Velocity)>) {
    let delta = time.delta_seconds();
    for (mut transform, velocity) in q.iter_mut() {
        transform.translation += Vec3::new(velocity.0.x, velocity.0.y, velocity.0.z) * delta;
    }
}

/// Component for things which rotate at a fixed speed
#[derive(Debug, Default, Component)]
pub struct Rotating(pub Quat);

pub fn apply_rotation(time: Res<Time>, mut q: Query<(&mut Transform, &Rotating)>) {
    let delta = time.delta_seconds();
    for (mut transform, Rotating(quat)) in q.iter_mut() {
        transform.rotate(*quat * delta);
    }
}

/// a glimmering effect to a PointLight
#[derive(Debug, Component)]
pub struct Glimmers {
    pub amplitude_min: f32,
    pub amplitude_max: f32,
}

impl Default for Glimmers {
    fn default() -> Self {
        Self {
            amplitude_min: 20.,
            amplitude_max: 100.,
        }
    }
}

pub fn apply_glimmer(time: Res<Time>, mut q: Query<(&mut PointLight, &Glimmers)>) {
    let time = time.elapsed_seconds_f64() as f32;
    for (mut light, glimmer) in q.iter_mut() {
        let amp = glimmer.amplitude_max - glimmer.amplitude_min;
        light.range = glimmer.amplitude_max - amp * (time * 2.).sin().abs();
    }
}

/// Component for entities that wobble a bit.
///
/// Entities with this component will always wobble around the origin position.
/// Position them first at the parent entity,
/// then attach entities with this component as children.
#[derive(Debug, Component)]
pub struct Wobbles {
    pub amplitude: f32,
    pub frequency: Vec2,
}

impl Default for Wobbles {
    fn default() -> Self {
        Self {
            amplitude: 0.075,
            frequency: Vec2::new(0.66, 1.),
        }
    }
}

pub fn apply_wobble(time: Res<LiveTime>, mut q: Query<(&mut Transform, &Wobbles)>) {
    let time = time.elapsed_seconds();
    for (mut transform, wobble) in q.iter_mut() {
        let offset = Vec3::new(
            wobble.amplitude * (wobble.frequency.x * time).sin(),
            wobble.amplitude * (wobble.frequency.y * time).cos(),
            0.0,
        );
        *transform = Transform::from_translation(offset);
    }
}

/// An effect that makes something fall to the ground
#[derive(Debug, Default, Component)]
pub struct Collapsing {
    pub speed: f32,
}

pub fn apply_collapse(time: Res<Time>, mut q: Query<(&mut Velocity, &mut Collapsing)>) {
    let delta = time.delta_seconds();
    for (mut velocity, mut collapsing) in q.iter_mut() {
        collapsing.speed += 168. * delta;
        velocity.0.y -= collapsing.speed * delta;
    }
}

/// Marker for an entity that clips Y to 0 if it goes below it
#[derive(Debug, Default, Component)]
pub struct StaysOnFloor;

pub fn stay_on_floor(mut q: Query<&mut Transform, With<StaysOnFloor>>) {
    for mut transform in q.iter_mut() {
        if transform.translation.y < 0. {
            transform.translation.y = 0.;
        }
    }
}

/// Make something despawn after a certain time (in seconds).
///
/// Does not despawn recursively.
#[derive(Debug, Component)]
pub struct TimeToLive(pub f32);

pub fn time_to_live(time: Res<Time>, mut cmd: Commands, mut q: Query<(Entity, &mut TimeToLive)>) {
    let delta = time.delta_seconds();
    for (entity, mut ttl) in q.iter_mut() {
        ttl.0 -= delta;
        if ttl.0 <= 0. {
            cmd.entity(entity).despawn();
        }
    }
}

/// Component to make something fade away
/// (reduces opacity to 0 over time)
#[derive(Debug, Default, Component)]
pub struct FadesAway;

pub fn fade_away(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q: Query<&Handle<StandardMaterial>, With<FadesAway>>,
) {
    let delta = time.delta_seconds();
    for material in q.iter_mut() {
        let Some(material) = materials.get_mut(material.id()) else {
            return;
        };
        let new_alpha = (material.base_color.alpha() - delta * 1.5).max(0.);
        material.base_color.set_alpha(new_alpha);
    }
}
