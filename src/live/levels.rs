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
    pub fn add_decision(&mut self, decision: Decision) -> bool {
        if self.stage >= LevelSpec::MAX_STAGES {
            warn!("Cannot move to the next level: maximum stage reached");
            return false;
        }

        if decision == Decision::Left {
            self.stage += 1;
        } else {
            self.decisions |= 1 << self.stage;
            self.stage += 1;
        }
        true
    }
}

/// Global resource for the current level
#[derive(Debug, Default, Resource)]
pub struct CurrentLevel {
    pub id: LevelId,
    pub spec: LevelSpec,
}

impl CurrentLevel {
    pub fn advance(&mut self, decision: Decision) -> bool {
        if self.id.add_decision(decision) {
            self.spec = LevelSpec::level(self.id);
            true
        } else {
            false
        }
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
    const MAX_STAGES: u8 = 5;

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
            // stage 4 xxx<
            level @ LevelId {
                stage: 4,
                decisions,
            } if (decisions >> 3) == 0 => Self::level_4l(level),
            // stage 4 xxx>
            level @ LevelId { stage: 4, .. } => Self::level_4r(level),

            // ending 2
            LevelId {
                stage: 5,
                // keep going left then right
                decisions: 0b10000,
            }
            | LevelId {
                stage: 5,
                // keep going right, then left
                decisions: 0b01111,
            } => Self::ending_bedroom(),

            // ending 3: zig-zag
            LevelId {
                stage: 5,
                // zig-zag
                decisions: 0b01010,
            }
            | LevelId {
                stage: 5,
                // zig-zag
                decisions: 0b10101,
            } => Self::ending_dungeon(),

            // ending 4: the mirror
            LevelId {
                stage: 5,
                decisions: 0b01001,
            } => Self::ending_mirror(),

            // fallback for most levels after the final stage
            // (this will depend on how many levels I mange to build...)
            LevelId { stage: 5, .. } => Self::ending_circle(),
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
                        (include_str!("./interludes/2_2.txt"), Some("interlude-cube.png")),
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
            rng_seed: 0x3333_3333_fefe + level.decisions as u64 * 997,
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

    fn level_2r(level: LevelId) -> Self {
        let harder = level.decisions == 0b11;

        let spawner_1 = if harder {
            MobSpawner::new(
                12,
                1.75,
                [10, 12, 15, 21, 25, 35, 49, 50, 54, 56, 63, 70, 72],
            )
        } else {
            MobSpawner::new(10, 2., [3, 5, 7, 10, 12, 15, 18, 21, 25, 27, 35])
        };

        let spawner_2 = if harder {
            MobSpawner::new(
                20,
                1.7,
                [
                    9, 10, 12, 15, 21, 25, 35, 49, 50, 54, 56, 60, 63, 70, 72, 84, 85, 87,
                ],
            )
        } else {
            MobSpawner::new(15, 1.8, [3, 5, 7, 9, 12, 14, 18, 24, 39, 49, 54, 56, 63])
        };

        let spawner_3 = if harder {
            MobSpawner::new(
                24,
                1.5,
                [
                    4, 9, 12, 15, 21, 24, 27, 32, 33, 35, 39, 45, 49, 50, 54, 55, 56, 70, 77, 81,
                    87, 91, 98,
                ],
            )
        } else {
            MobSpawner::new(
                20,
                1.75,
                [2, 3, 4, 5, 6, 9, 12, 15, 21, 24, 32, 45, 49, 50, 51, 55],
            )
        };

        LevelSpec {
            corridor_length: 180.,
            rng_seed: 0xc36b_58ca_1297_c528 + level.decisions as u64 * 997,
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
                        x: 0.,
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
                (0.36, spawner_1).into(),
                (0.4, spawner_2).into(),
                // add cube 2
                (
                    0.6,
                    ThingKind::WeaponCube {
                        x: 0.,
                        num: 2.into(),
                    },
                )
                    .into(),
                // a stronger mob spawner
                (0.75, spawner_3).into(),
            ],
        }
    }

    fn level_2l(level: LevelId) -> Self {
        let harder = level.decisions == 0b01;

        let spawner_1 = if !harder {
            MobSpawner::new(10, 2., [4, 6, 7, 8, 12, 14, 16, 24, 35, 36])
        } else {
            MobSpawner::new(
                15,
                1.8,
                [
                    6, 7, 8, 12, 16, 20, 24, 28, 32, 35, 36, 49, 54, 56, 63, 64, 77, 80,
                ],
            )
        };

        let spawner_2 = if !harder {
            MobSpawner::new(
                15,
                1.8,
                [6, 7, 12, 18, 28, 20, 32, 35, 40, 42, 49, 54, 66, 70],
            )
        } else {
            MobSpawner::new(
                18,
                1.75,
                [
                    12, 14, 18, 20, 28, 32, 35, 54, 64, 40, 42, 63, 66, 70, 77, 80, 84, 88, 91, 96,
                    98,
                ],
            )
        };

        let spawner_3 = if !harder {
            MobSpawner::new(
                18,
                1.7,
                [6, 7, 8, 11, 12, 7, 8, 16, 22, 21, 24, 32, 36, 49, 55, 63],
            )
        } else {
            MobSpawner::new(
                24,
                1.6,
                [
                    6, 12, 7, 8, 11, 16, 21, 22, 24, 32, 33, 36, 49, 55, 56, 60, 64, 70, 77, 84,
                    91, 99, 121,
                ],
            )
        };

        LevelSpec {
            corridor_length: 180.,
            rng_seed: 0x3434_3434_1297_c528 + level.decisions as u64 * 997,
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
                (0.3, spawner_1).into(),
                (0.35, spawner_2).into(),
                // add cube 11
                (
                    0.65,
                    ThingKind::WeaponCube {
                        x: 0.,
                        num: 11.into(),
                    },
                )
                    .into(),
                // a stronger mob spawner
                (0.75, spawner_3).into(),
            ],
        }
    }

    fn level_3(level: LevelId) -> Self {
        let harder = level.decisions == 0b001;

        let spawner_1 = MobSpawner::new(
            12,
            2.,
            [
                // 1/3
                frac!(1 / 3),
                frac!(2 / 6),
                frac!(3 / 9),
                frac!(6 / 18),
                // 1/4
                frac!(1 / 4),
                frac!(4 / 16),
                frac!(5 / 20),
                frac!(6 / 24),
            ],
        );

        let spawner_2 = if !harder {
            MobSpawner::new(
                15,
                1.9,
                [
                    // 1/3
                    frac!(1 / 3),
                    frac!(2 / 6),
                    frac!(4 / 12),
                    frac!(6 / 18),
                    frac!(7 / 21),
                    frac!(8 / 24),
                    frac!(9 / 27),
                    // 1/4
                    frac!(1 / 4),
                    frac!(3 / 12),
                    frac!(5 / 20),
                    frac!(6 / 24),
                    frac!(7 / 28),
                    frac!(8 / 32),
                    frac!(9 / 36),
                ],
            )
        } else {
            MobSpawner::new(
                18,
                1.72,
                [
                    // 1/3
                    frac!(4 / 12),
                    frac!(6 / 18),
                    frac!(7 / 21),
                    frac!(8 / 24),
                    frac!(9 / 27),
                    frac!(12 / 36),
                    frac!(15 / 45),
                    // 1/4
                    frac!(3 / 12),
                    frac!(5 / 20),
                    frac!(6 / 24),
                    frac!(7 / 28),
                    frac!(8 / 32),
                    frac!(9 / 36),
                    frac!(12 / 48),
                    frac!(15 / 60),
                ],
            )
        };

        let spawner_3 = if !harder {
            MobSpawner::new(
                20,
                1.8,
                [
                    // 1/3
                    frac!(1 / 3),
                    frac!(2 / 6),
                    frac!(3 / 9),
                    frac!(4 / 12),
                    frac!(6 / 18),
                    frac!(7 / 21),
                    // 1/4
                    frac!(1 / 4),
                    frac!(6 / 24),
                    frac!(8 / 32),
                    frac!(9 / 36),
                    // 3/4
                    frac!(3 / 4),
                    frac!(6 / 8),
                    frac!(9 / 12),
                    frac!(12 / 16),
                    frac!(15 / 20),
                    frac!(18 / 24),
                ],
            )
        } else {
            MobSpawner::new(
                25,
                1.7,
                [
                    // 1/3
                    frac!(2 / 6),
                    frac!(3 / 9),
                    frac!(4 / 12),
                    frac!(6 / 18),
                    frac!(7 / 21),
                    frac!(9 / 27),
                    frac!(10 / 30),
                    frac!(16 / 48),
                    frac!(24 / 72),
                    // 1/4
                    frac!(3 / 12),
                    frac!(6 / 24),
                    frac!(8 / 32),
                    frac!(9 / 36),
                    frac!(10 / 40),
                    frac!(12 / 48),
                    frac!(24 / 96),
                    // 3/4
                    frac!(3 / 4),
                    frac!(6 / 8),
                    frac!(9 / 12),
                    frac!(12 / 16),
                    frac!(15 / 20),
                    frac!(18 / 24),
                    frac!(21 / 28),
                    frac!(24 / 32),
                    frac!(33 / 44),
                ],
            )
        };

        // the level where we start having fractions
        LevelSpec {
            corridor_length: 180.,
            rng_seed: 0x3454_4321_ffff + level.decisions as u64 * 997,
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
                (0.225, spawner_1).into(),
                // another mob spawner wave
                (0.3, spawner_2).into(),
                // spawn a 3/4 cube
                (
                    0.5,
                    ThingKind::WeaponCube {
                        x: 0.5,
                        num: frac!(3 / 4),
                    },
                )
                    .into(),
                // a heavier spawner
                (0.72, spawner_3).into(),
            ],
        }
    }

    fn level_4r(level: LevelId) -> Self {
        // the hardest level
        let harder = level.decisions == 0b1001;

        let spawner_1 = MobSpawner::new(
            14,
            1.7,
            [
                // 1/2
                frac!(1 / 2),
                frac!(2 / 4),
                frac!(4 / 8),
                frac!(6 / 12),
                frac!(8 / 16),
                frac!(10 / 20),
                frac!(16 / 32),
                // 1/5
                frac!(3 / 15),
                frac!(4 / 20),
                frac!(5 / 25),
                frac!(6 / 30),
                frac!(7 / 35),
                frac!(8 / 40),
                // 1/7
                frac!(1 / 7),
                frac!(2 / 14),
                frac!(3 / 21),
                frac!(4 / 28),
                frac!(5 / 35),
                frac!(6 / 42),
                frac!(7 / 49),
                frac!(8 / 56),
                frac!(9 / 63),
                frac!(11 / 77),
                // 1/8
                frac!(1 / 8),
                frac!(2 / 16),
                frac!(3 / 24),
                frac!(4 / 32),
                frac!(5 / 40),
                frac!(6 / 48),
                frac!(7 / 56),
                frac!(8 / 64),
                frac!(9 / 72),
            ],
        );

        let spawner_2 = MobSpawner::new(
            16,
            1.7,
            [
                // 1/2
                frac!(1 / 2),
                frac!(2 / 4),
                frac!(4 / 8),
                frac!(6 / 12),
                frac!(8 / 16),
                frac!(10 / 20),
                frac!(16 / 32),
                // 1/5
                frac!(3 / 15),
                frac!(4 / 20),
                frac!(5 / 25),
                frac!(6 / 30),
                frac!(7 / 35),
                frac!(8 / 40),
                // 1/7
                frac!(1 / 7),
                frac!(2 / 14),
                frac!(3 / 21),
                frac!(4 / 28),
                frac!(5 / 35),
                frac!(6 / 42),
                frac!(7 / 49),
                frac!(8 / 56),
                frac!(9 / 63),
                frac!(11 / 77),
                // 1/8
                frac!(1 / 8),
                frac!(2 / 16),
                frac!(3 / 24),
                frac!(4 / 32),
                frac!(5 / 40),
                frac!(6 / 48),
                frac!(7 / 56),
                frac!(8 / 64),
                frac!(9 / 72),
            ],
        );

        let spawner_3 = if !harder {
            MobSpawner::new(
                25,
                1.68,
                [
                    // 2
                    frac!(4 / 2),
                    frac!(12 / 6),
                    frac!(18 / 9),
                    frac!(24 / 12),
                    // 1/5
                    frac!(2 / 10),
                    frac!(5 / 25),
                    frac!(6 / 30),
                    frac!(7 / 35),
                    frac!(10 / 50),
                    // 1/7
                    frac!(1 / 7),
                    frac!(2 / 14),
                    frac!(3 / 21),
                    frac!(6 / 42),
                    frac!(8 / 56),
                    // 1/8
                    frac!(1 / 8),
                    frac!(2 / 16),
                    frac!(3 / 24),
                    frac!(5 / 40),
                    frac!(6 / 48),
                    frac!(8 / 64),
                ],
            )
        } else {
            MobSpawner::new(
                32,
                1.55,
                [
                    // 2
                    frac!(4 / 2),
                    frac!(12 / 6),
                    frac!(18 / 9),
                    frac!(24 / 12),
                    frac!(16 / 4),
                    frac!(120, 1),
                    // 1/5
                    frac!(2 / 10),
                    frac!(5 / 25),
                    frac!(6 / 30),
                    frac!(7 / 35),
                    frac!(8 / 40),
                    frac!(10 / 50),
                    frac!(11 / 55),
                    // 1/7
                    frac!(3 / 21),
                    frac!(4 / 28),
                    frac!(6 / 42),
                    frac!(8 / 56),
                    // 1/8
                    frac!(2 / 16),
                    frac!(3 / 24),
                    frac!(5 / 40),
                    frac!(6 / 48),
                    frac!(7 / 56),
                    frac!(8 / 64),
                    frac!(9 / 72),
                    frac!(12 / 96),
                    // 7/8
                    frac!(14 / 16),
                    frac!(21 / 24),
                    frac!(28 / 32),
                    frac!(35 / 40),
                    frac!(42 / 48),
                    frac!(49 / 56),
                    frac!(56 / 64),
                    frac!(63 / 72),
                    frac!(84 / 96),
                ],
            )
        };

        let mut out = LevelSpec {
            corridor_length: 250.,
            rng_seed: 0x1ab2_4547_fdab,
            things: vec![
                // spawn 4 fraction cubes
                (
                    0.09,
                    ThingKind::WeaponCube {
                        x: 1.,
                        num: frac!(1 / 2),
                    },
                )
                    .into(),
                (
                    0.12,
                    ThingKind::WeaponCube {
                        x: 0.5,
                        num: frac!(1 / 5),
                    },
                )
                    .into(),
                (
                    0.15,
                    ThingKind::WeaponCube {
                        x: 0.,
                        num: frac!(1 / 7),
                    },
                )
                    .into(),
                (
                    0.18,
                    ThingKind::WeaponCube {
                        x: -0.5,
                        num: frac!(1 / 8),
                    },
                )
                    .into(),
                // spawn a mob spawner
                (0.26, spawner_1).into(),
                // spawn another mob spawner
                (0.32, spawner_2).into(),
                // spawn a 2 cube
                (
                    0.55,
                    ThingKind::WeaponCube {
                        x: 0.,
                        num: 2.into(),
                    },
                )
                    .into(),
                // final mob spawner
                (0.7, spawner_3).into(),
            ],
        };

        if harder {
            // also spawn a 7/8 cube
            out.things.push(
                (
                    0.5,
                    ThingKind::WeaponCube {
                        x: 0.5,
                        num: frac!(7 / 8),
                    },
                )
                    .into(),
            );
        }

        out
    }

    fn level_4l(level: LevelId) -> Self {
        LevelSpec {
            corridor_length: 250.,
            rng_seed: 0x5c98_a112_fabf_551d + level.decisions as u64 * 997,
            things: vec![
                // spawn 4 fraction cubes
                (
                    0.09,
                    ThingKind::WeaponCube {
                        x: 1.,
                        num: frac!(1 / 3),
                    },
                )
                    .into(),
                (
                    0.12,
                    ThingKind::WeaponCube {
                        x: 0.5,
                        num: frac!(1 / 4),
                    },
                )
                    .into(),
                (
                    0.15,
                    ThingKind::WeaponCube {
                        x: 0.,
                        num: frac!(1 / 5),
                    },
                )
                    .into(),
                (
                    0.18,
                    ThingKind::WeaponCube {
                        x: -0.5,
                        num: frac!(1 / 6),
                    },
                )
                    .into(),
                // spawn a mob spawner with equivalent fractions
                (
                    0.275,
                    MobSpawner::new(
                        22,
                        1.72,
                        [
                            // 1/3
                            frac!(1 / 3),
                            frac!(2 / 6),
                            frac!(3 / 9),
                            frac!(4 / 12),
                            frac!(5 / 15),
                            frac!(6 / 18),
                            frac!(7 / 21),
                            frac!(8 / 24),
                            // 1/4
                            frac!(1 / 4),
                            frac!(2 / 8),
                            frac!(3 / 12),
                            frac!(4 / 16),
                            frac!(5 / 20),
                            frac!(6 / 24),
                            frac!(7 / 28),
                            frac!(8 / 32),
                            // 1/5
                            frac!(1 / 5),
                            frac!(2 / 10),
                            frac!(3 / 15),
                            frac!(4 / 20),
                            frac!(5 / 25),
                            frac!(6 / 30),
                            frac!(7 / 35),
                            frac!(8 / 40),
                            // 1/6
                            frac!(1 / 6),
                            frac!(2 / 12),
                            frac!(3 / 18),
                            frac!(4 / 24),
                            frac!(5 / 30),
                            frac!(6 / 36),
                            frac!(7 / 42),
                            frac!(8 / 48),
                        ],
                    ),
                )
                    .into(),
                // spawn a 2 cube
                (
                    0.5,
                    ThingKind::WeaponCube {
                        x: 0.,
                        num: 2.into(),
                    },
                )
                    .into(),
                // mob spawner
                (
                    0.7,
                    MobSpawner::new(
                        24,
                        1.7,
                        [
                            // 2
                            frac!(4 / 2),
                            frac!(12 / 6),
                            frac!(18 / 9),
                            frac!(16 / 4),
                            frac!(24 / 12),
                            frac!(48, 1),
                            // 1/3
                            frac!(1 / 3),
                            frac!(2 / 6),
                            frac!(3 / 9),
                            frac!(4 / 12),
                            frac!(5 / 15),
                            frac!(6 / 18),
                            frac!(7 / 21),
                            frac!(8 / 24),
                            frac!(9 / 27),
                            // 1/4
                            frac!(1 / 4),
                            frac!(2 / 8),
                            frac!(3 / 12),
                            frac!(4 / 16),
                            frac!(5 / 20),
                            frac!(6 / 24),
                            frac!(7 / 28),
                            frac!(8 / 32),
                            frac!(9 / 36),
                            // 1/5
                            frac!(1 / 5),
                            frac!(2 / 10),
                            frac!(3 / 15),
                            frac!(4 / 20),
                            frac!(9 / 45),
                            frac!(10 / 50),
                            frac!(11 / 55),
                            // 1/6
                            frac!(1 / 6),
                            frac!(2 / 12),
                            frac!(3 / 18),
                            frac!(4 / 24),
                            frac!(5 / 30),
                            frac!(6 / 36),
                            frac!(7 / 42),
                            frac!(8 / 48),
                            frac!(9 / 54),
                            frac!(10 / 60),
                            frac!(11 / 66),
                        ],
                    ),
                )
                    .into(),
            ],
        }
    }

    fn ending_circle() -> Self {
        // Ending 1: walk in circles
        Self::ending_level_impl(vec![
            (include_str!("interludes/z_circle_1.txt"), None),
            (include_str!("interludes/z_circle_2.txt"), None),
        ])
    }

    fn ending_bedroom() -> Self {
        // Ending 2: the bedroom
        Self::ending_level_impl(vec![
            (
                include_str!("interludes/z_bedroom_1.txt"),
                Some("interlude-bedroom.png"),
            ),
            (include_str!("interludes/z_bedroom_2.txt"), None),
            (include_str!("interludes/z_bedroom_3.txt"), None),
        ])
    }

    fn ending_dungeon() -> LevelSpec {
        // Ending 3: the dungeon
        Self::ending_level_impl(vec![
            (
                include_str!("interludes/z_dungeon_1.txt"),
                Some("interlude-dungeon-1.png"),
            ),
            (
                include_str!("interludes/z_dungeon_2.txt"),
                Some("interlude-dungeon-2.png"),
            ),
            (include_str!("interludes/z_dungeon_3.txt"), None),
            (include_str!("interludes/z_dungeon_4.txt"), None),
        ])
    }

    fn ending_mirror() -> LevelSpec {
        // Ending 4: the mirror
        Self::ending_level_impl(vec![
            (include_str!("interludes/z_mirror_1.txt"), None),
            (include_str!("interludes/z_mirror_2.txt"), None),
            (
                include_str!("interludes/z_mirror_3.txt"),
                Some("interlude-mirror-1.png"),
            ),
            (include_str!("interludes/z_mirror_4.txt"), None),
            (
                include_str!("interludes/z_mirror_5.txt"),
                Some("interlude-mirror-2.png"),
            ),
        ])
    }

    /// helper function for levels which just end the game
    fn ending_level_impl(
        interludes: impl IntoIterator<Item = (&'static str, Option<&'static str>)>,
    ) -> Self {
        LevelSpec {
            corridor_length: 1000.,
            rng_seed: 0,
            things: vec![(0., InterludeSpec::from_sequence_and_exit(interludes)).into()],
        }
    }
}
