//! Tests des déviations I18 et Fab 4.

use super::*;
use crate::counters::{HI_LO, HI_OPT_II, ZEN};
use crate::rules::Rules;

fn ctx(cards: [u8; 2], dealer_up: u8, tc: f64) -> DecisionContext {
    let mut hand = crate::hand::Hand::new(false);
    hand.add(cards[0]);
    hand.add(cards[1]);
    let (total, is_soft) = hand.value();
    DecisionContext {
        total,
        is_soft,
        pair_rank: hand.pair_rank(),
        dealer_up,
        can_double: true,
        can_split: true,
        can_surrender: true,
        true_count: tc,
    }
}

fn hilo(level: DeviationLevel) -> DeviationStrategy {
    DeviationStrategy::new(BasicStrategy::new(&Rules::default()), level, &HI_LO)
}

#[test]
fn at_or_above_rules() {
    let s = hilo(DeviationLevel::I18);
    // 12 v 3 : tirer en basic, rester si TC ≥ +2
    assert_eq!(s.decide(&ctx([10, 2], 3, 1.0)), PlayerAction::Hit);
    assert_eq!(s.decide(&ctx([10, 2], 3, 2.0)), PlayerAction::Stand);
    // 11 v A : doubler si TC ≥ +1
    assert_eq!(s.decide(&ctx([6, 5], ACE, 0.0)), PlayerAction::Hit);
    assert_eq!(s.decide(&ctx([6, 5], ACE, 1.0)), PlayerAction::Double);
    // 10,10 v 6 : split si TC ≥ +4
    assert_eq!(s.decide(&ctx([10, 10], 6, 3.0)), PlayerAction::Stand);
    assert_eq!(s.decide(&ctx([10, 10], 6, 4.0)), PlayerAction::Split);
    // 15 v 10 sans abandon possible : rester si TC ≥ +4
    let mut c = ctx([10, 5], TEN, 4.0);
    c.can_surrender = false;
    assert_eq!(s.decide(&c), PlayerAction::Stand);
    c.true_count = 3.0;
    assert_eq!(s.decide(&c), PlayerAction::Hit);
}

#[test]
fn below_rules() {
    let s = hilo(DeviationLevel::I18);
    // 13 v 2 : rester par défaut, tirer si TC < −1
    assert_eq!(s.decide(&ctx([10, 3], 2, -1.0)), PlayerAction::Stand);
    assert_eq!(s.decide(&ctx([10, 3], 2, -2.0)), PlayerAction::Hit);
    // 12 v 4 : tirer si TC < 0
    assert_eq!(s.decide(&ctx([10, 2], 4, 0.0)), PlayerAction::Stand);
    assert_eq!(s.decide(&ctx([10, 2], 4, -1.0)), PlayerAction::Hit);
}

#[test]
fn double_deviation_falls_back_to_hit() {
    let s = hilo(DeviationLevel::I18);
    let mut c = ctx([6, 4], TEN, 5.0);
    c.can_double = false;
    assert_eq!(s.decide(&c), PlayerAction::Hit);
}

#[test]
fn fab4_surrender_both_directions() {
    let s = hilo(DeviationLevel::I18Fab4);
    // 15 v 10 : abandon si TC ≥ 0, sinon tirer (pas d'abandon basic)
    assert_eq!(s.decide(&ctx([10, 5], TEN, 0.0)), PlayerAction::Surrender);
    assert_eq!(s.decide(&ctx([10, 5], TEN, -1.0)), PlayerAction::Hit);
    // 14 v 10 : abandon si TC ≥ +3
    assert_eq!(s.decide(&ctx([9, 5], TEN, 3.0)), PlayerAction::Surrender);
    assert_eq!(s.decide(&ctx([9, 5], TEN, 2.0)), PlayerAction::Hit);
    // 15 v A : abandon si TC ≥ +1
    assert_eq!(s.decide(&ctx([10, 5], ACE, 1.0)), PlayerAction::Surrender);
    assert_eq!(s.decide(&ctx([10, 5], ACE, 0.0)), PlayerAction::Hit);
    // Sans Fab 4, 15 v 10 reste un abandon basic quel que soit le compte
    let i18 = hilo(DeviationLevel::I18);
    assert_eq!(
        i18.decide(&ctx([10, 5], TEN, -3.0)),
        PlayerAction::Surrender
    );
    // 8,8 v 10 : le split reste prioritaire
    assert_eq!(s.decide(&ctx([8, 8], TEN, 5.0)), PlayerAction::Split);
}

#[test]
fn insurance_by_level() {
    assert!(!hilo(DeviationLevel::None).take_insurance(10.0));
    assert!(hilo(DeviationLevel::Insurance).take_insurance(3.0));
    assert!(!hilo(DeviationLevel::Insurance).take_insurance(2.0));
    assert!(hilo(DeviationLevel::I18Fab4).take_insurance(3.0));
}

#[test]
fn none_level_is_basic_strategy() {
    let s = hilo(DeviationLevel::None);
    assert_eq!(s.decide(&ctx([10, 2], 3, 10.0)), PlayerAction::Hit);
    assert_eq!(s.decide(&ctx([10, 6], TEN, 10.0)), PlayerAction::Surrender);
}

#[test]
fn indices_are_scaled_for_level_two_systems() {
    let basic = BasicStrategy::new(&Rules::default());
    let zen = DeviationStrategy::new(basic, DeviationLevel::I18, &ZEN);
    // Index d'assurance Hi-Lo +3 × 1,79 ≈ +5
    assert!(!zen.take_insurance(4.0));
    assert!(zen.take_insurance(5.0));
    // 12 v 3 : +2 × 1,79 ≈ +4
    assert_eq!(zen.decide(&ctx([10, 2], 3, 3.0)), PlayerAction::Hit);
    assert_eq!(zen.decide(&ctx([10, 2], 3, 4.0)), PlayerAction::Stand);
    let hopt = DeviationStrategy::new(basic, DeviationLevel::I18, &HI_OPT_II);
    assert!(hopt.take_insurance(5.0) && !hopt.take_insurance(4.0));
    assert_eq!(scale_index(0.0, 1.79), 0.0);
    assert_eq!(scale_index(-2.0, 1.79), -4.0);
}
