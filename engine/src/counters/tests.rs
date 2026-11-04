//! Tests des systèmes de comptage.

use super::*;
use crate::cards::TEN;

fn count(system: &'static CountSystem, n_decks: u8, cards: &[u8]) -> TagCounter {
    let mut counter = TagCounter::new(system, n_decks, DEFAULT_ACE_FACTOR);
    for &c in cards {
        counter.observe(c);
    }
    counter
}

#[test]
fn full_deck_sums() {
    for system in ALL_SYSTEMS {
        let expected = if system.balanced { 0.0 } else { 4.0 };
        assert_eq!(system.full_deck_sum(), expected, "{}", system.id);
    }
}

#[test]
fn ko_initial_running_count() {
    assert_eq!(KO.initial_running_count(6), -20.0);
    assert_eq!(KO.initial_running_count(1), 0.0);
    assert_eq!(HI_LO.initial_running_count(6), 0.0);
}

#[test]
fn running_counts_on_known_sequence() {
    // Séquence de l'annexe pédagogique : 3, 5, K, 7, Q, A, 8, 5, 4, 2
    let seq = [3, 5, TEN, 7, TEN, ACE, 8, 5, 4, 2];
    let expected = [
        (&HI_LO, 2.0),
        (&KO, -20.0 + 3.0),
        (
            &HI_OPT_II,
            1.0 + 2.0 - 2.0 + 1.0 - 2.0 + 0.0 + 0.0 + 2.0 + 2.0 + 1.0,
        ),
        (
            &OMEGA_II,
            1.0 + 2.0 - 2.0 + 1.0 - 2.0 + 0.0 + 0.0 + 2.0 + 2.0 + 1.0,
        ),
        (
            &WONG_HALVES,
            1.0 + 1.5 - 1.0 + 0.5 - 1.0 - 1.0 + 0.0 + 1.5 + 1.0 + 0.5,
        ),
        (
            &ZEN,
            1.0 + 2.0 - 2.0 + 1.0 - 2.0 - 1.0 + 0.0 + 2.0 + 2.0 + 1.0,
        ),
    ];
    for (system, rc) in expected {
        assert_eq!(count(system, 6, &seq).running_count(), rc, "{}", system.id);
    }
    // Hi-Lo : +2 avec 4 jeux restants → true count +0,5
    assert_eq!(count(&HI_LO, 6, &seq).true_count(4.0), 0.5);
}

#[test]
fn ko_true_count_is_balanced_equivalent() {
    // Sabot neuf : compte équilibré nul
    let fresh = TagCounter::new(&KO, 6, DEFAULT_ACE_FACTOR);
    assert_eq!(fresh.true_count(6.0), 0.0);
    assert_eq!(fresh.betting_count(6.0), -20.0);
    // Après un jeu complet distribué, le running count vaut −16 et le compte
    // équilibré revient à 0
    let mut deck = Vec::new();
    for rank in 1..=10u8 {
        deck.extend(std::iter::repeat_n(rank, copies_per_deck(rank) as usize));
    }
    let after = count(&KO, 6, &deck);
    assert_eq!(after.running_count(), -16.0);
    assert!(after.true_count(5.0).abs() < 1e-12);
}

#[test]
fn ace_side_count_corrects_betting_count_only() {
    // Deux As vus en début de sabot : le reste est pauvre en As
    let poor = count(&HI_OPT_II, 6, &[ACE, ACE]);
    let dr = 310.0 / 52.0;
    assert_eq!(poor.running_count(), 0.0);
    assert_eq!(poor.true_count(dr), 0.0);
    let expected = DEFAULT_ACE_FACTOR * (22.0 - 4.0 * dr) / dr;
    assert!((poor.betting_count(dr) - expected).abs() < 1e-12);
    assert!(poor.betting_count(dr) < 0.0);
    // Sans side-count, compte de mise = true count
    let zen = count(&ZEN, 6, &[ACE, ACE]);
    assert_eq!(zen.betting_count(dr), zen.true_count(dr));
}

#[test]
fn reset_restores_irc() {
    let mut ko = count(&KO, 6, &[2, 3, 4]);
    ko.reset(6);
    assert_eq!(ko.running_count(), -20.0);
    let mut hilo = count(&HI_LO, 6, &[2, 3, ACE]);
    hilo.reset(6);
    assert_eq!(hilo.running_count(), 0.0);
}

#[test]
fn scale_factors_vs_hi_lo() {
    assert!((HI_LO.scale_vs_hi_lo() - 1.0).abs() < 1e-12);
    assert!((HI_OPT_II.scale_vs_hi_lo() - (112.0f64 / 40.0).sqrt()).abs() < 1e-12);
    assert!((ZEN.scale_vs_hi_lo() - (128.0f64 / 40.0).sqrt()).abs() < 1e-12);
    assert!(system_by_id("wong-halves").is_some());
    assert!(system_by_id("inconnu").is_none());
}
