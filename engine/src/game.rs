//! Déroulé d'une main de blackjack.
//!
//! Ordre : distribution, assurance, peek du croupier, décisions du joueur (splits
//! inclus), jeu du croupier, résolution. Chaque carte est transmise au compteur au
//! moment où elle devient visible ; la hole card n'est comptée qu'à sa révélation.

// Importation des modules internes
use crate::cards::{Shoe, ACE, TEN};
use crate::counters::Counter;
use crate::hand::Hand;
use crate::rules::Rules;
use crate::strategy::{DecisionContext, PlayerAction, PlayingStrategy};

/// Nombre maximal de mains après splits (3 splits = 4 mains).
const MAX_HANDS: usize = 4;
/// Borne de sécurité des décisions par main : une main ne peut pas recevoir
/// plus de 21 cartes avant de dépasser 21, chaque action en tire au plus une.
const MAX_ACTIONS_PER_HAND: usize = 24;

/// Résultat d'une main initiale (splits et doubles compris).
///
/// Attributs principaux :
/// * `net` - gain net, assurance comprise, dans l'unité de la mise ;
/// * `initial_bet` - mise initiale ;
/// * `total_wagered` - somme des mises engagées sur les mains (hors assurance).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoundOutcome {
    /// Gain net de la main initiale (assurance comprise).
    pub net: f64,
    /// Mise initiale.
    pub initial_bet: f64,
    /// Mises totales engagées (doubles et splits compris, assurance exclue).
    pub total_wagered: f64,
}

/// État d'une main du joueur pendant la résolution.
#[derive(Debug, Clone, Copy, Default)]
struct SeatHand {
    hand: Hand,
    wager: f64,
    surrendered: bool,
    split_aces: bool,
}

/// Contexte mutable partagé pendant une main.
struct Table<'a, S, C> {
    shoe: &'a mut Shoe,
    rules: &'a Rules,
    strategy: &'a S,
    counter: &'a mut C,
    hands: [SeatHand; MAX_HANDS],
    n_hands: usize,
    dealer_up: u8,
}

impl<S: PlayingStrategy, C: Counter> Table<'_, S, C> {
    /// Tire une carte visible et la transmet au compteur.
    fn deal_visible(&mut self) -> u8 {
        let card = self.shoe.draw();
        self.counter.observe(card);
        card
    }

    /// True count de décision, tronqué vers −∞.
    fn decision_count(&self) -> f64 {
        self.counter.true_count(self.shoe.decks_remaining()).floor()
    }

    fn context(&self, idx: usize) -> DecisionContext {
        let seat = &self.hands[idx];
        let hand = &seat.hand;
        let (total, is_soft) = hand.value();
        let pair_rank = hand.pair_rank();
        let ace_locked = seat.split_aces && !self.rules.resplit_aces;
        let can_split = pair_rank.is_some()
            && self.n_hands <= usize::from(self.rules.max_splits)
            && !ace_locked;
        let no_more_cards = seat.split_aces && !self.rules.hit_split_aces;
        let can_double = hand.n_cards() == 2
            && !no_more_cards
            && (!hand.from_split() || self.rules.double_after_split)
            && (self.rules.double_any_two || (9..=11).contains(&total));
        let can_surrender = self.rules.late_surrender
            && self.rules.dealer_peeks
            && self.n_hands == 1
            && hand.n_cards() == 2
            && !hand.from_split();
        DecisionContext {
            total,
            is_soft,
            pair_rank,
            dealer_up: self.dealer_up,
            can_double,
            can_split,
            can_surrender,
            true_count: self.decision_count(),
        }
    }

    /// Sépare la main `idx` : la seconde carte part dans une nouvelle main
    /// insérée juste après, jouée immédiatement après la main courante.
    fn split(&mut self, idx: usize) {
        debug_assert!(
            self.n_hands < MAX_HANDS,
            "split au-delà de {MAX_HANDS} mains"
        );
        let rank = self.hands[idx].hand.first_card();
        let wager = self.hands[idx].wager;
        let mut fresh = SeatHand {
            hand: Hand::new(true),
            wager,
            surrendered: false,
            split_aces: rank == ACE,
        };
        fresh.hand.add(rank);
        for j in (idx + 1..=self.n_hands).rev() {
            self.hands[j] = self.hands[j - 1];
        }
        self.hands[idx] = fresh;
        self.hands[idx + 1] = fresh;
        self.n_hands += 1;
        let card = self.deal_visible();
        self.hands[idx].hand.add(card);
    }

    /// Joue la main `idx` jusqu'à ce qu'elle soit terminée.
    fn play_hand(&mut self, idx: usize) {
        // Une main issue d'un split n'a encore qu'une carte quand vient son tour
        if self.hands[idx].hand.n_cards() == 1 {
            let card = self.deal_visible();
            self.hands[idx].hand.add(card);
        }
        for _ in 0..MAX_ACTIONS_PER_HAND {
            if self.hands[idx].hand.total() >= 21 {
                return;
            }
            let ctx = self.context(idx);
            let locked_aces = self.hands[idx].split_aces && !self.rules.hit_split_aces;
            let action = self.strategy.decide(&ctx);
            match action {
                PlayerAction::Split if ctx.can_split => self.split(idx),
                // As splittés sans tirage : seul un resplit est encore possible
                _ if locked_aces => return,
                PlayerAction::Stand | PlayerAction::Split => return,
                PlayerAction::Surrender if ctx.can_surrender => {
                    self.hands[idx].surrendered = true;
                    return;
                }
                PlayerAction::Double if ctx.can_double => {
                    self.hands[idx].wager *= 2.0;
                    let card = self.deal_visible();
                    self.hands[idx].hand.add(card);
                    return;
                }
                PlayerAction::Hit | PlayerAction::Double | PlayerAction::Surrender => {
                    let card = self.deal_visible();
                    self.hands[idx].hand.add(card);
                }
            }
        }
        unreachable!("main non terminée après {MAX_ACTIONS_PER_HAND} actions");
    }

    /// Le croupier tire jusqu'à 17 (soft 17 selon H17/S17).
    fn play_dealer(&mut self, dealer: &mut Hand) {
        for _ in 0..MAX_ACTIONS_PER_HAND {
            let (total, soft) = dealer.value();
            let hits_soft_17 = total == 17 && soft && self.rules.dealer_hits_soft_17;
            if total >= 17 && !hits_soft_17 {
                return;
            }
            let card = self.deal_visible();
            dealer.add(card);
        }
        unreachable!("croupier non terminé après {MAX_ACTIONS_PER_HAND} cartes");
    }

    /// Gain net des mains du joueur face à la main finale du croupier.
    fn settle(&self, dealer: &Hand) -> f64 {
        let dealer_total = dealer.total();
        let dealer_bj = dealer.is_blackjack();
        self.hands[..self.n_hands]
            .iter()
            .map(|seat| {
                if seat.surrendered {
                    // Sans peek, un abandon face à un blackjack perd toute la mise
                    return if dealer_bj {
                        -seat.wager
                    } else {
                        -seat.wager / 2.0
                    };
                }
                let total = seat.hand.total();
                if seat.hand.is_bust() || dealer_bj {
                    -seat.wager
                } else if dealer.is_bust() || total > dealer_total {
                    seat.wager
                } else if total < dealer_total {
                    -seat.wager
                } else {
                    0.0
                }
            })
            .sum()
    }
}

/// Joue une main initiale complète.
///
/// # Arguments
///
/// * `shoe` - Sabot mélangé ; il n'est jamais remélangé ici.
/// * `rules` - Règles de la table.
/// * `strategy` - Stratégie de jeu injectée.
/// * `counter` - Compteur informé de chaque carte visible.
/// * `bet` - Mise initiale (> 0).
///
/// # Returns
///
/// Le résultat net de la main et les mises engagées.
pub fn play_round<S: PlayingStrategy, C: Counter>(
    shoe: &mut Shoe,
    rules: &Rules,
    strategy: &S,
    counter: &mut C,
    bet: f64,
) -> RoundOutcome {
    assert!(bet > 0.0, "la mise doit être > 0 (reçu {bet})");

    // ------------------------------------------------------------------
    // 1. Distribution : joueur, carte visible, joueur, hole card cachée
    // ------------------------------------------------------------------
    let mut table = Table {
        shoe,
        rules,
        strategy,
        counter,
        hands: [SeatHand::default(); MAX_HANDS],
        n_hands: 1,
        dealer_up: 0,
    };
    table.hands[0].wager = bet;
    let p1 = table.deal_visible();
    let up = table.deal_visible();
    let p2 = table.deal_visible();
    let hole = table.shoe.draw();
    table.hands[0].hand.add(p1);
    table.hands[0].hand.add(p2);
    table.dealer_up = up;
    let mut dealer = Hand::new(false);
    dealer.add(up);
    dealer.add(hole);

    // ------------------------------------------------------------------
    // 2. Assurance, décidée par le compte avant le peek
    // ------------------------------------------------------------------
    let insurance = if up == ACE && table.strategy.take_insurance(table.decision_count()) {
        bet / 2.0
    } else {
        0.0
    };
    let insurance_net = if insurance > 0.0 {
        if dealer.is_blackjack() {
            2.0 * insurance
        } else {
            -insurance
        }
    } else {
        0.0
    };

    // ------------------------------------------------------------------
    // 3. Peek : un blackjack croupier termine la main sur la mise initiale
    // ------------------------------------------------------------------
    let peeked = rules.dealer_peeks && (up == ACE || up == TEN);
    if peeked && dealer.is_blackjack() {
        table.counter.observe(hole);
        let main_net = if table.hands[0].hand.is_blackjack() {
            0.0
        } else {
            -bet
        };
        return outcome(main_net + insurance_net, bet, bet);
    }
    if table.hands[0].hand.is_blackjack() {
        table.counter.observe(hole);
        let main_net = if dealer.is_blackjack() {
            0.0
        } else {
            bet * rules.blackjack_payout
        };
        return outcome(main_net + insurance_net, bet, bet);
    }

    // ------------------------------------------------------------------
    // 4. Décisions du joueur, main par main (les splits insèrent des mains)
    // ------------------------------------------------------------------
    let mut idx = 0;
    while idx < table.n_hands {
        table.play_hand(idx);
        idx += 1;
    }

    // ------------------------------------------------------------------
    // 5. Jeu du croupier : la hole card est révélée, il ne tire que si une
    //    main du joueur reste en jeu
    // ------------------------------------------------------------------
    table.counter.observe(hole);
    let any_live = table.hands[..table.n_hands]
        .iter()
        .any(|s| !s.surrendered && !s.hand.is_bust());
    if any_live {
        table.play_dealer(&mut dealer);
    }

    // ------------------------------------------------------------------
    // 6. Résolution
    // ------------------------------------------------------------------
    let wagered: f64 = table.hands[..table.n_hands].iter().map(|s| s.wager).sum();
    let net = table.settle(&dealer) + insurance_net;
    outcome(net, bet, wagered)
}

fn outcome(net: f64, initial_bet: f64, total_wagered: f64) -> RoundOutcome {
    RoundOutcome {
        net,
        initial_bet,
        total_wagered,
    }
}

#[cfg(test)]
mod tests;
