//! The live action module, containing the active game logic
use bevy::prelude::*;

mod mob;
mod projectile;
mod weapon;

/// Component for things with a health meter.
#[derive(Debug, Component)]
pub struct Health {
    pub value: f32,
    pub max: f32,
}

/// Marker for the player
#[derive(Debug, Component)]
pub struct Player;

/// The plugin which adds everything related to the live action
pub struct LiveActionPlugin;

impl Plugin for LiveActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                mob::destroy_spawner_when_done,
                projectile::apply_velocity,
                projectile::spawn_projectiles_via_mouseclick,
            ),
        );
    }
}
