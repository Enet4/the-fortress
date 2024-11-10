//! Components and systems for various visual effects
use bevy::prelude::*;

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

pub fn apply_wobble(time: Res<Time>, mut q: Query<(&mut Transform, &Wobbles)>) {
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

pub fn apply_collapse(time: Res<Time>, mut q: Query<(&mut Transform, &mut Collapsing)>) {
    let delta = time.delta_seconds();
    for (mut transform, mut collapsing) in q.iter_mut() {
        if transform.translation.y <= 0. {
            continue;
        }

        collapsing.speed += 4. * delta;
        transform.translation.y = (transform.translation.y - collapsing.speed * delta).max(0.);
    }
}
