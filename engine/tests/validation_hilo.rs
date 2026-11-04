//! Validation Hi-Lo contre les ordres de grandeur de *Blackjack Attack*.
//!
//! Cibles (6 jeux, S17, DAS, LS, I18 + Fab 4) :
//! * spread 1-15, pénétration 4,5/6 : ≈ +0,83 % ;
//! * spread 1-5, pénétration 4/6 : ≈ +0,16 %.
//!
//! Ces chiffres dépendent de la rampe exacte utilisée par la référence ; la rampe
//! du projet est `bet = clamp(1, spread, floor(TC − 1))`. Les écarts mesurés sont
//! documentés dans `docs/METHODOLOGIE.md` : la cible 1-5 est vérifiée à ± 0,15 %,
//! la cible 1-15 n'est pas atteignable avec une rampe de pente 1 (écart affiché).
//!
//! Lancement : `cargo test --release -p engine --test validation_hilo -- --ignored --nocapture`.

// Importation des modules internes
use engine::betting::UnitRamp;
use engine::counters::HI_LO;
use engine::deviations::DeviationLevel;
use engine::rules::Rules;
use engine::simulation::{derive_shoe_seeds, run_simulation, BetModel, SimulationConfig};

const RUNS: usize = 400;
const HANDS_PER_RUN: u64 = 100_000;
const TOLERANCE_PCT: f64 = 0.15;

fn measure(spread: f64, penetration: f64) -> (f64, f64) {
    let config = SimulationConfig {
        rules: Rules {
            penetration,
            ..Rules::default()
        },
        system: Some(&HI_LO),
        deviations: DeviationLevel::I18Fab4,
        bet_model: BetModel::Units(UnitRamp::new(&HI_LO, spread, 1.0)),
        n_hands_per_run: HANDS_PER_RUN,
        equity_sample_interval: HANDS_PER_RUN,
        ace_factor: 2.0,
        n_threads: std::thread::available_parallelism().map_or(4, |n| n.get()),
    };
    let res = run_simulation(&config, &derive_shoe_seeds(0x41_7AC4, RUNS)).expect("simulation");
    let m = res.units.metrics(1000.0).expect("métriques");
    let stderr = 100.0 * m.std_per_hand / (m.n_hands_simulated as f64).sqrt() / m.avg_bet;
    println!(
        "Hi-Lo 1-{spread} pén. {penetration:.4} : {} mains, avantage {:.4} % (± {stderr:.4} %), mise moy. {:.3}, σ {:.3}",
        m.n_hands_simulated, m.advantage_pct, m.avg_bet, m.std_per_hand
    );
    (m.advantage_pct, stderr)
}

#[test]
#[ignore = "validation lourde (40 M mains par config)"]
fn hi_lo_spread_15_deep_penetration() {
    // Écart documenté : avec une rampe de pente 1 unité par TC, le spread 1-15
    // n'est atteint qu'à TC +16 et l'avantage plafonne vers +0,6 % quel que soit
    // ramp_offset (mesures dans METHODOLOGIE.md). La cible +0,83 % suppose une
    // rampe plus raide. On vérifie ici l'ordre de grandeur et la cohérence avec
    // le spread 1-5, et on affiche l'écart à la référence.
    let (adv_15, stderr) = measure(15.0, 0.75);
    let (adv_5, _) = measure(5.0, 0.75);
    println!("écart à la référence +0,83 % : {:+.4} %", adv_15 - 0.83);
    assert!(
        adv_15 > 4.0 * stderr,
        "avantage 1-15 non significatif : {adv_15:.4} %"
    );
    assert!(
        adv_15 > adv_5,
        "le spread 1-15 doit battre le spread 1-5 à pénétration égale"
    );
}

#[test]
#[ignore = "validation lourde (40 M mains par config)"]
fn hi_lo_spread_5_shallow_penetration() {
    let (adv, _) = measure(5.0, 4.0 / 6.0);
    assert!(
        (adv - 0.16).abs() <= TOLERANCE_PCT,
        "avantage {adv:.4} % hors de 0,16 ± {TOLERANCE_PCT} %"
    );
}
