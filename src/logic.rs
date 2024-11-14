//! Module for the logic of player attacks based on mathematical rules.
//!
//! The player has to perform an attack using a number
//! which is considered _effective_ against the number of the target
//! (usually a mob or some other target).
//!
//! Effective attacks will mutate the target number
//! or damage the target.
//! When damaged, its health decreases.
//! If it reaches zero, the target is destroyed.
//! Otherwise,
//! in special cases where the target is more robust,
//! a new number may be regenerated.
use crate::live::Target;

pub type Num = num_rational::Ratio<i16>;

/// The rule for damaging the target.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TargetRule {
    /// The default rule for most cases.
    /// The attack number must be a factor of the target,
    /// further decomposing the target number until it reaches 1.
    ///
    /// If the target number is 1,
    /// any attack will damage it.
    /// Otherwise, an attack of 1, 0, or a non-whole number
    /// is a failed attack.
    #[default]
    Factorize,
    /// The number must be exactly equal to the target.
    ///
    /// Equivalent fractions are considered equal.
    Equal,
    /// Any attack will fail.
    /// This is usually a temporary state or a rule for temporary obstacles.
    Invulnerable,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AttackTest {
    /// The attack was effective,
    /// the target becomes the given number (`Some`)
    /// or is damaged (`None`).
    Effective(Option<Num>),
    /// The attack was ineffective.
    Failed,
}

#[inline]
pub fn test_attack_on(target: &Target, attack: Num) -> AttackTest {
    test_attack(target.rule, attack, target.num)
}

/// Test an attack to see what effect it has on the target.
pub fn test_attack(rule: TargetRule, attack: Num, target: Num) -> AttackTest {
    match rule {
        TargetRule::Equal => {
            if attack == target {
                AttackTest::Effective(None)
            } else {
                AttackTest::Failed
            }
        }
        TargetRule::Factorize => {
            if target == Num::ONE || target == attack {
                AttackTest::Effective(None)
            } else if !attack.is_integer() {
                AttackTest::Failed
            } else if target % attack == Num::ZERO {
                AttackTest::Effective(Some(target / attack))
            } else {
                AttackTest::Failed
            }
        }
        TargetRule::Invulnerable => AttackTest::Failed,
    }
}
