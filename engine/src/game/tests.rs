//! Tests du déroulé d'une main sur des sabots empilés.

use super::*;
use crate::counters::NullCounter;
use crate::strategy::BasicStrategy;

/// Compteur de test qui mémorise les cartes observées.
#[derive(Default)]
struct Recorder {
    seen: Vec<u8>,
}

impl Counter for Recorder {
    fn reset(&mut self, _n_decks: u8) {
        self.seen.clear();
    }
    fn observe(&mut self, card: u8) {
        self.seen.push(card);
    }
    fn running_count(&self) -> f64 {
        0.0
    }
    fn true_count(&self, _decks_remaining: f64) -> f64 {
        0.0
    }
    fn betting_count(&self, _decks_remaining: f64) -> f64 {
        0.0
    }
}

/// Ordre de distribution : joueur 1, croupier visible, joueur 2, hole card, puis tirages.
fn play(top: &[u8]) -> RoundOutcome {
    play_with(top, Rules::default()).0
}

fn play_with(top: &[u8], rules: Rules) -> (RoundOutcome, Vec<u8>) {
    let mut shoe = Shoe::new(rules.n_decks, rules.penetration);
    shoe.stack_top(top);
    let strategy = BasicStrategy::new(&rules);
    let mut recorder = Recorder::default();
    let out = play_round(&mut shoe, &rules, &strategy, &mut recorder, 1.0);
    (out, recorder.seen)
}

#[test]
fn player_blackjack_pays_three_to_two() {
    let out = play(&[ACE, 9, 10, 7]);
    assert_eq!(out.net, 1.5);
    assert_eq!(out.total_wagered, 1.0);
}

#[test]
fn dealer_blackjack_after_peek_takes_initial_bet_only() {
    let (out, seen) = play_with(&[10, ACE, 6, 10], Rules::default());
    assert_eq!(out.net, -1.0);
    // La hole card est observée une fois révélée
    assert_eq!(seen, vec![10, ACE, 6, 10]);
    assert_eq!(play(&[ACE, ACE, 10, 10]).net, 0.0);
}

#[test]
fn hole_card_is_observed_only_when_revealed() {
    // 10,7 v 9 : rester ; croupier 9 + 8 = 17
    let (out, seen) = play_with(&[10, 9, 7, 8], Rules::default());
    assert_eq!(out.net, 0.0);
    assert_eq!(seen, vec![10, 9, 7, 8]);
}

#[test]
fn double_doubles_the_wager() {
    // 6,5 v 6 : double, reçoit 10 → 21 ; croupier 6,10 tire 10 → bust
    let out = play(&[6, 6, 5, 10, 10, 10]);
    assert_eq!(out.net, 2.0);
    assert_eq!(out.total_wagered, 2.0);
}

#[test]
fn late_surrender_loses_half() {
    // 10,6 v 10 : abandon (le croupier n'a pas de blackjack)
    let out = play(&[10, 10, 6, 7]);
    assert_eq!(out.net, -0.5);
}

#[test]
fn split_eights_plays_two_hands() {
    // 8,8 v 6 : split ; main 1 reçoit 3 (11 → double, reçoit 10 = 21),
    // main 2 reçoit 10 (18, reste) ; croupier 6,10 tire 10 → bust
    let out = play(&[8, 6, 8, 10, 3, 10, 10, 10]);
    assert_eq!(out.total_wagered, 3.0);
    assert_eq!(out.net, 3.0);
}

#[test]
fn split_aces_receive_one_card_and_no_blackjack() {
    // A,A v 9 : split ; chaque As reçoit un 10 (21 non naturel), croupier 9,7 tire 4 → 20
    let out = play(&[ACE, 9, ACE, 7, 10, 10, 4]);
    assert_eq!(out.total_wagered, 2.0);
    assert_eq!(out.net, 2.0);
    // Un As splitté qui reçoit un 2 ne peut plus tirer : 13 contre 20 perd
    let out = play(&[ACE, 9, ACE, 7, 2, 10, 4]);
    assert_eq!(out.net, 0.0);
}

#[test]
fn resplits_are_capped_at_four_hands() {
    // 8,8 v 6 avec des 8 en série : au plus 4 mains, ensuite 16 dur reste v 6
    let top = [8, 6, 8, 10, 8, 8, 8, 8, 8, 10, 10, 10, 10];
    let mut shoe = Shoe::new(6, 0.75);
    shoe.stack_top(&top);
    let rules = Rules::default();
    let out = play_round(
        &mut shoe,
        &rules,
        &BasicStrategy::new(&rules),
        &mut NullCounter,
        1.0,
    );
    assert_eq!(out.total_wagered, 4.0);
}

#[test]
fn dealer_hits_soft_17_only_in_h17() {
    // 10,8 v 6 ; croupier 6,A = soft 17
    let top = [10, 6, 8, ACE, 3];
    let s17 = play_with(&top, Rules::default()).0;
    assert_eq!(s17.net, 1.0);
    let h17 = play_with(
        &top,
        Rules {
            dealer_hits_soft_17: true,
            ..Rules::default()
        },
    )
    .0;
    // H17 : le croupier tire 3 → 20, le 18 perd
    assert_eq!(h17.net, -1.0);
}
