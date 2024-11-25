//! Module for level components and specifications
use std::fmt::{self, Write};

use bevy::prelude::*;

use crate::logic::Num;

use super::{interlude::InterludeSpec, mob::MobSpawner, Decision};

/// Level identifier.
///
/// Levels are identified by an increasing stage number
/// starting from 0,
/// and a small bit vector of decisions made up to that stage,
/// starting from the least significant bit,
/// where 0 is left and 1 is right.
/// For example, `LevelId { stage: 2, decision: 0b10 }`
/// means third stage where the player went left then right.
///
/// This allows for the game to provide unique level quirks
/// based on the decisions made in the previous levels.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct LevelId {
    pub stage: u8,
    pub decisions: u8,
}

impl fmt::Display for LevelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.stage)?;
        if self.stage == 0 {
            return Ok(());
        } else {
            f.write_char('(')?;
            for i in 0..self.stage {
                let d = (self.decisions >> i) & 1;
                if d != 0 {
                    f.write_char('>')?;
                } else {
                    f.write_char('<')?;
                }
            }
            f.write_char(')')?;
            Ok(())
        }
    }
}

impl LevelId {
    pub fn decision_at(&self, stage: u8) -> Option<Decision> {
        if stage >= self.stage {
            return None;
        }
        let d = (self.decisions >> stage) & 1;
        if d != 0 {
            Some(Decision::Right)
        } else {
            Some(Decision::Left)
        }
    }

    pub fn add_decision(&mut self, decision: Decision) {
        if self.stage > 7 {
            warn!("Cannot move to the next level: maximum stage reached");
            return;
        }

        if decision == Decision::Left {
            self.stage += 1;
        } else {
            self.decisions |= 1 << self.stage;
            self.stage += 1;
        }
    }
}

/// Global resource for the current level
#[derive(Debug, Default, Resource)]
pub struct CurrentLevel {
    pub id: LevelId,
    pub spec: LevelSpec,
}

impl CurrentLevel {
    pub fn advance(&mut self, decision: Decision) {
        self.id.add_decision(decision);
        self.spec = LevelSpec::level(self.id);
    }

    pub fn reset(&mut self) {
        *self = CurrentLevel::default();
    }
}

/// Generic thing in a level
/// to be placed relative to the corridor length
#[derive(Debug)]
pub struct Thing {
    /// position relative to the length of the corridor,
    /// from 0 (start) to 1 (end)
    pub at: f32,
    /// what should be at that position
    pub what: ThingKind,
}

impl<T> From<(f32, T)> for Thing
where
    T: Into<ThingKind>,
{
    fn from(value: (f32, T)) -> Self {
        Thing {
            at: value.0,
            what: value.1.into(),
        }
    }
}

/// The actual thing that should appear in the level
#[derive(Debug)]
pub enum ThingKind {
    WeaponCube { x: f32, num: Num },
    MobSpawner(MobSpawner),
    Interlude(InterludeSpec),
    Dread,
    MoveOn,
}

impl From<MobSpawner> for ThingKind {
    fn from(value: MobSpawner) -> Self {
        ThingKind::MobSpawner(value)
    }
}

impl From<InterludeSpec> for ThingKind {
    fn from(value: InterludeSpec) -> Self {
        ThingKind::Interlude(value)
    }
}

#[derive(Debug)]
pub struct LevelSpec {
    pub corridor_length: f32,
    /// the things in the level
    pub things: Vec<Thing>,
}

impl Default for LevelSpec {
    fn default() -> Self {
        Self::level_0()
    }
}

impl LevelSpec {
    fn level(level: LevelId) -> Self {
        match level {
            // starting level
            LevelId { stage: 0, .. } => Self::level_0(),
            LevelId { stage: 1, .. } => Self::level_carp(),
            level @ LevelId { stage: 2, .. } => todo!("Unspecified level {level}"),
            level @ LevelId { stage: 3, .. } => todo!("Unspecified level {level}"),
            level @ LevelId { stage: 4, .. } => todo!("Unspecified level {level}"),
            level @ LevelId { stage: 5, .. } => todo!("Unspecified level {level}"),
            level @ LevelId { stage: 6, .. } => todo!("Unspecified level {level}"),
            level @ LevelId { stage: 7, .. } => todo!("Unspecified level {level}"),
            _ => unreachable!("Unexpected level {level}"),
        }
    }

    fn level_0() -> Self {
        LevelSpec {
            corridor_length: 150.,
            things: vec![
                // starting story
                (
                    0.,
                    InterludeSpec::from_sequence([
                        (include_str!("./interludes/1_1.txt"), Some("interlude-01.png")),
                        (include_str!("./interludes/1_2.txt"), Some("interlude-02.png")),
                    ])
                ).into(),

                // teeny jumpscare
                (
                    0.3,
                    ThingKind::Dread,
                ).into(),

                // message from the wizard
                (
                    0.425,
                    InterludeSpec::from_sequence([
                        (include_str!("./interludes/2_1.txt"), None),
                        (include_str!("./interludes/2_2.txt"), None),
                    ])
                ).into(),

                // recover from dread
                (
                    0.5,
                    ThingKind::MoveOn,
                ).into(),

                // add a weapon cube
                (
                    0.5,
                    ThingKind::WeaponCube { x: 0., num: 2.into() }
                ).into(),

                // add a mob spawner that spawns a single mob
                (
                    0.62,
                    MobSpawner::new(1, 2., vec![2.into()]),
                ).into(),

                // an interlude just before the fork
                (
                    0.95,
                    InterludeSpec::from_sequence([
                        (
                            "At the end of the corridor, you see two possible paths. There appear to be no distinct visual cues between the two.",
                            None,
                        ),
                        ("Reluctantly, you tap into your precognitive skills, and choose.", None),
                    ]),
                ).into()
            ],
        }
    }

    #[deprecated(note = "just for testing purposes, get rid of this before releasing")]
    fn level_carp() -> Self {
        Self::exit_level_impl([("You went right and died.\n\nThe End.", None)])
    }

    /// helper function for levels which just end the game
    fn exit_level_impl(
        interludes: impl IntoIterator<Item = (&'static str, Option<&'static str>)>,
    ) -> Self {
        LevelSpec {
            corridor_length: 1000.,
            things: vec![(0., InterludeSpec::from_sequence_and_exit(interludes)).into()],
        }
    }
}
