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
    /// the seed defining reproducible behavior patterns in the level
    pub rng_seed: u64,
    /// the things in the level
    pub things: Vec<Thing>,
}

impl Default for LevelSpec {
    fn default() -> Self {
        Self::level_0()
    }
}

macro_rules! frac {
    ($a: literal / $b: literal) => {
        Num::new_raw($a, $b)
    };
    ($a: literal, $b: literal) => {
        Num::new_raw($a, $b)
    };
}

impl LevelSpec {
    fn level(level: LevelId) -> Self {
        match level {
            // starting level
            LevelId { stage: 0, .. } => Self::level_0(),
            // stage 1
            LevelId { stage: 1, .. } => Self::level_1(level),
            // stage 2 x<
            level @ LevelId {
                stage: 2,
                decisions,
            } if (decisions >> 1) == 0 => Self::level_2l(level),
            // stage 2 x>
            level @ LevelId { stage: 2, .. } => Self::level_2r(level),
            // stage 3
            LevelId { stage: 3, .. } => Self::level_3(level),
            // TODO other levels
            level @ LevelId { stage: 4, .. } => Self::level_carp(),
            level @ LevelId { stage: 5, .. } => Self::level_carp(),
            level @ LevelId { stage: 6, .. } => Self::level_carp(),
            // fallback for most levels after the final stage
            // (this will depend on how many levels I mange to build...)
            level @ LevelId { stage: 7, .. } => Self::ending_circle(),
            _ => unreachable!("Unexpected level {level}"),
        }
    }

    fn level_0() -> Self {
        LevelSpec {
            corridor_length: 150.,
            rng_seed: 0x01,
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

                // add a mob spawner that spawns a few mobs
                (
                    0.62,
                    MobSpawner::new(5, 2., [2]),
                ).into(),

                // add a mob spawner that spawns a single mob
                (
                    0.7,
                    MobSpawner::new(1, 2., [6]),
                ).into(),

                // an interlude just before the fork
                (
                    0.95,
                    InterludeSpec::from_sequence([
                        (
                            "At the end of the corridor, you see two possible paths. There appear to be no remarkable differences between the two.",
                            None,
                        ),
                        ("Reluctantly, you tap into your precognitive skills, and choose.", None),
                    ]),
                ).into()
            ],
        }
    }

    fn level_1(level: LevelId) -> Self {
        LevelSpec {
            corridor_length: 200.,
            rng_seed: 0x3333_3333,
            things: vec![
                // another message
                (
                    0.1,
                    InterludeSpec::from_sequence([
                        (include_str!("./interludes/3_1.txt"), None),
                    ])
                ).into(),

                // give two cubes to the player
                (
                    0.15,
                    ThingKind::WeaponCube { x: 1., num: 2.into() }
                ).into(),
                (
                    0.2,
                    ThingKind::WeaponCube { x: -1., num: 3.into() }
                ).into(),

                // mob spawner with 2s and 3s
                (
                    0.4,
                    MobSpawner::new(10, 2., [2, 3]),
                ).into(),

                // a bit more difficult
                (
                    0.65,
                    MobSpawner::new(16, 1.75, [2, 3, 4, 6, 9]),
                ).into(),

                // another interlude just before the fork
                (
                    0.95,
                    InterludeSpec::from_sequence([
                        (
                            "You see another fork up ahead. Given the complete lack of guidance or clues, you feel like you will have to search every path until you find the wizard.",
                            None,
                        ),
                        (if level.decisions == 0 {
                            "You went left before. Which way should you go this time?"
                        } else {
                            "You went right before. Which way should you go this time?"
                        }, None),
                    ]),
                ).into()
            ],
        }
    }

    fn level_2l(level: LevelId) -> Self {
        LevelSpec {
            corridor_length: 180.,
            rng_seed: 0x3333_3333,
            things: vec![
                // give three cubes to the player
                (
                    0.09,
                    ThingKind::WeaponCube {
                        x: 1.,
                        num: 3.into(),
                    },
                )
                    .into(),
                (
                    0.1,
                    ThingKind::WeaponCube {
                        x: -1.,
                        num: 5.into(),
                    },
                )
                    .into(),
                (
                    0.12,
                    ThingKind::WeaponCube {
                        x: -1.,
                        num: 7.into(),
                    },
                )
                    .into(),
                // one mob spawner after another
                (
                    0.36,
                    MobSpawner::new(10, 1.75, [10, 7, 25, 28, 39, 49, 50, 56]),
                )
                    .into(),
                (
                    0.4,
                    MobSpawner::new(15, 1.75, [3, 9, 14, 20, 21, 24, 39, 45, 63]),
                )
                    .into(),
                // add cube 2
                (
                    0.6,
                    ThingKind::WeaponCube {
                        x: 0.,
                        num: 2.into(),
                    },
                )
                    .into(),
                // use two mob spawners
                (0.75, MobSpawner::new(10, 2.5, [2, 3, 4, 6, 9, 12])).into(),
                (
                    0.75,
                    MobSpawner::new(10, 1.75, [21, 24, 32, 12, 45, 49, 50, 56, 91]),
                )
                    .into(),
            ],
        }
    }

    fn level_2r(level: LevelId) -> Self {
        LevelSpec {
            corridor_length: 180.,
            rng_seed: 0x3434_3434,
            things: vec![
                // give three cubes to the player
                (
                    0.09,
                    ThingKind::WeaponCube {
                        x: 1.,
                        num: 4.into(),
                    },
                )
                    .into(),
                (
                    0.1,
                    ThingKind::WeaponCube {
                        x: 0.,
                        num: 6.into(),
                    },
                )
                    .into(),
                (
                    0.12,
                    ThingKind::WeaponCube {
                        x: -1.,
                        num: 7.into(),
                    },
                )
                    .into(),
                // one mob spawner after another
                (
                    0.36,
                    MobSpawner::new(10, 1.75, [8, 7, 16, 24, 36, 49, 56, 80]),
                )
                    .into(),
                (
                    0.4,
                    MobSpawner::new(
                        15,
                        1.75,
                        [6, 12, 14, 20, 28, 32, 39, 45, 54, 63, 64, 70, 66],
                    ),
                )
                    .into(),
                // add cube 11
                (
                    0.6,
                    ThingKind::WeaponCube {
                        x: 0.,
                        num: 11.into(),
                    },
                )
                    .into(),
                // use two mob spawners
                (0.75, MobSpawner::new(10, 2.5, [6, 7, 8, 11, 16, 60])).into(),
                (
                    0.75,
                    MobSpawner::new(10, 1.75, [12, 21, 24, 32, 36, 49, 55, 56, 64, 121]),
                )
                    .into(),
            ],
        }
    }

    fn level_3(level: LevelId) -> Self {
        // the level where we start having fractions
        LevelSpec {
            corridor_length: 180.,
            rng_seed: 0x3454_4321,
            things: vec![
                // spawn a 1/3 cube
                (
                    0.1,
                    ThingKind::WeaponCube {
                        x: 0.5,
                        num: frac!(1 / 3),
                    },
                )
                    .into(),
                // spawn a 1/4 cube
                (
                    0.15,
                    ThingKind::WeaponCube {
                        x: -0.5,
                        num: frac!(1 / 4),
                    },
                )
                    .into(),
                // spawn a mob spawner with equivalent fractions
                (
                    0.3,
                    MobSpawner::new(
                        20,
                        1.75,
                        [
                            // 1/3
                            frac!(1 / 3),
                            frac!(2 / 6),
                            frac!(6 / 18),
                            frac!(16 / 48),
                            // 1/4
                            frac!(1 / 4),
                            frac!(4 / 16),
                            frac!(6 / 24),
                            frac!(8 / 32),
                        ],
                    ),
                )
                    .into(),
                // spawn a 3/4 cube
                (
                    0.5,
                    ThingKind::WeaponCube {
                        x: 0.5,
                        num: frac!(3 / 4),
                    },
                )
                    .into(),
                // two spawners
                (
                    0.7,
                    MobSpawner::new(
                        12,
                        2.,
                        [
                            // 1/3
                            frac!(1 / 3),
                            frac!(6 / 18),
                            frac!(16 / 48),
                            frac!(10 / 30),
                            // 1/4
                            frac!(1 / 4),
                            frac!(6 / 24),
                            frac!(8 / 32),
                            // 3/4
                            frac!(3 / 4),
                            frac!(9 / 12),
                            frac!(33 / 44),
                        ],
                    ),
                )
                    .into(),
                (
                    0.7,
                    MobSpawner::new(
                        14,
                        1.75,
                        [
                            // 1/3
                            frac!(1 / 3),
                            frac!(2 / 6),
                            frac!(4 / 12),
                            frac!(7 / 21),
                            // 1/4
                            frac!(1 / 4),
                            frac!(4 / 16),
                            frac!(6 / 24),
                            frac!(8 / 32),
                            frac!(10 / 40),
                            // 3/4
                            frac!(6 / 8),
                            frac!(15 / 40),
                            frac!(21 / 28),
                        ],
                    ),
                )
                    .into(),
            ],
        }
    }

    fn ending_circle() -> Self {
        // Ending 1
        LevelSpec {
            corridor_length: 1000.,
            rng_seed: 0,
            things: vec![(
                0.,
                InterludeSpec::from_sequence_and_exit([
                    (include_str!("interludes/z_circle_1.txt"), None),
                    (include_str!("interludes/z_circle_2.txt"), None),
                ]),
            )
                .into()],
        }
    }

    #[deprecated(note = "just for testing purposes, get rid of this before releasing")]
    fn level_carp() -> Self {
        Self::exit_level_impl([("You went down a cliff.\n\nThe End.", None)])
    }

    /// helper function for levels which just end the game
    fn exit_level_impl(
        interludes: impl IntoIterator<Item = (&'static str, Option<&'static str>)>,
    ) -> Self {
        LevelSpec {
            corridor_length: 1000.,
            rng_seed: 0,
            things: vec![(0., InterludeSpec::from_sequence_and_exit(interludes)).into()],
        }
    }
}
