//! Module for holding phase triggers.
use bevy::prelude::*;

/// Component for triggers which activate when the player reaches a certain Z coordinate.
///
/// It is expected that this trigger is removed after it is activated.
#[derive(Debug, Component)]
pub struct PhaseTrigger {
    /// at which position in the Z coordinate should the trigger be activated
    pub at_z: f32,
}

impl PhaseTrigger {
    /// Create a phase trigger
    /// given the length of the corridor
    /// and a relative position from 0 to 1
    pub fn new_by_corridor(corridor_length: f32, ratio: f32) -> Self {
        Self {
            at_z: (corridor_length - 6.) * ratio,
        }
    }

    pub fn should_trigger(&self, player_translate: &Vec3) -> bool {
        player_translate.z >= self.at_z
    }
}
