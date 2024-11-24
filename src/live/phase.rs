//! Module for holding phase triggers.
use bevy::prelude::*;

use crate::{assets::AudioHandles, postprocess::PostProcessSettings};

use super::player::{Player, PlayerMovement};

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

/// Custom effect to create a sense of dread.
#[derive(Debug, Component)]
pub struct Dread;

pub fn process_approach_dread(
    mut cmd: Commands,
    mut player_q: Query<(&Transform, &mut PlayerMovement), With<Player>>,
    trigger_q: Query<(Entity, &PhaseTrigger), With<Dread>>,
    mut postprocess_settings_q: Query<&mut PostProcessSettings>,
    audio_handles: Res<AudioHandles>,
) {
    let Ok((player_transform, mut player_movement)) = player_q.get_single_mut() else {
        return;
    };

    for (entity, trigger) in &trigger_q {
        if trigger.should_trigger(&player_transform.translation) {
            // set postprocessing to the max
            let Ok(mut postprocess_settings) = postprocess_settings_q.get_single_mut() else {
                continue;
            };
            postprocess_settings.intensity = 1.;

            // play dread sound
            audio_handles.play_dread(&mut cmd);

            // slow the player down a bit
            *player_movement = PlayerMovement::Slower;

            // remove entity entirely, no longer needed
            cmd.entity(entity).despawn();
        }
    }
}

/// Custom effect to make the player walk at normal speed,
/// recovering from the dread.
#[derive(Debug, Component)]
pub struct MoveOn;

pub fn process_approach_move_on(
    mut cmd: Commands,
    mut player_q: Query<(&Transform, &mut PlayerMovement), With<Player>>,
    trigger_q: Query<(Entity, &PhaseTrigger), With<MoveOn>>,
) {
    let Ok((player_transform, mut player_movement)) = player_q.get_single_mut() else {
        return;
    };

    for (entity, trigger) in &trigger_q {
        if trigger.should_trigger(&player_transform.translation) {
            // recover
            *player_movement = PlayerMovement::Walking;

            // remove entity entirely, no longer needed
            cmd.entity(entity).despawn();
        }
    }
}
