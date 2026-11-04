//! Validation bloquante de la basic strategy contre Wizard of Odds.
//!
//! Référence : House Edge Calculator de Wizard of Odds
//! (<https://wizardofodds.com/games/blackjack/calculator/>), 6 jeux, S17, DAS,
//! double sur deux cartes, resplit à 4 mains, pas de RSA, pas de tirage sur As
//! splittés, perte de la mise initiale seule face au blackjack (peek), late
//! surrender : 0,330512 % (stratégie optimale) + 0,0231 % (correction « basic
//! strategy with cut card » pour 6 jeux) = 0,353612 % d'avantage maison.
//!
//! Lancement : `cargo test --release -p engine --test validation_basic -- --ignored`.

// Importation des crates externes
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use rayon::prelude::*;

// Importation des modules internes
use engine::cards::Shoe;
use engine::counters::NullCounter;
use engine::game::play_round;
use engine::rules::Rules;
use engine::strategy::BasicStrategy;

/// Avantage joueur attendu (en %) d'après Wizard of Odds.
const WOO_PLAYER_EDGE_PCT: f64 = -0.353612;
const RUNS: u64 = 200;
const HANDS_PER_RUN: u64 = 125_000;

/// Somme du gain net et du gain net au carré sur une série de mains.
fn simulate_run(seed: u64, rules: &Rules) -> (f64, f64) {
    let strategy = BasicStrategy::new(rules);
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
    let mut shoe = Shoe::new(rules.n_decks, rules.penetration);
    shoe.shuffle(&mut rng);
    let mut counter = NullCounter;
    let (mut sum, mut sum_sq) = (0.0, 0.0);
    for _ in 0..HANDS_PER_RUN {
        if shoe.needs_reshuffle() {
            shoe.shuffle(&mut rng);
        }
        let out = play_round(&mut shoe, rules, &strategy, &mut counter, 1.0);
        sum += out.net;
        sum_sq += out.net * out.net;
    }
    (sum, sum_sq)
}

#[test]
#[ignore = "validation lourde (25 M mains), lancer avec --release -- --ignored"]
fn basic_strategy_matches_wizard_of_odds() {
    let rules = Rules::default();
    let partials: Vec<(f64, f64)> = (0..RUNS)
        .into_par_iter()
        .map(|i| simulate_run(0xB1AC_0000 + i, &rules))
        .collect();
    let (sum, sum_sq) = partials
        .iter()
        .fold((0.0, 0.0), |acc, p| (acc.0 + p.0, acc.1 + p.1));
    let n = (RUNS * HANDS_PER_RUN) as f64;
    let mean = sum / n;
    let sigma = (sum_sq / n - mean * mean).sqrt();
    let edge_pct = 100.0 * mean;
    let stderr_pct = 100.0 * sigma / n.sqrt();
    println!(
        "basic strategy : {n} mains, avantage {edge_pct:.4} % (± {stderr_pct:.4} %), σ/main {sigma:.4}"
    );
    assert!(
        (edge_pct - WOO_PLAYER_EDGE_PCT).abs() <= 0.05,
        "avantage {edge_pct:.4} % hors de {WOO_PLAYER_EDGE_PCT} ± 0,05 %"
    );
    assert!(
        (sigma - 1.15).abs() <= 0.02,
        "σ/main {sigma:.4} hors de 1,15 ± 0,02"
    );
}
