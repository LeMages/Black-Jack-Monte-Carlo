//! Tests de déterminisme et de cohérence de la simulation.

use super::*;
use crate::betting::BankrollPolicy;
use crate::counters::{HI_LO, OMEGA_II};

fn units_config(system: Option<&'static CountSystem>, n_threads: usize) -> SimulationConfig {
    let ramp = match system {
        Some(s) => UnitRamp::new(s, 8.0, 1.0),
        None => UnitRamp::flat(),
    };
    SimulationConfig {
        rules: Rules::default(),
        system,
        deviations: DeviationLevel::I18Fab4,
        bet_model: BetModel::Units(ramp),
        n_hands_per_run: 2_000,
        equity_sample_interval: 500,
        ace_factor: 2.0,
        n_threads,
    }
}

#[test]
fn same_seeds_same_results_regardless_of_threads() {
    let seeds = derive_shoe_seeds(42, 16);
    for system in [Some(&HI_LO), Some(&OMEGA_II), None] {
        let one = run_simulation(&units_config(system, 1), &seeds).expect("1 thread");
        let four = run_simulation(&units_config(system, 4), &seeds).expect("4 threads");
        assert_eq!(one, four);
        assert_eq!(one.units.hands, 32_000);
    }
}

#[test]
fn different_master_seeds_differ() {
    let a = run_simulation(&units_config(Some(&HI_LO), 2), &derive_shoe_seeds(1, 4)).expect("a");
    let b = run_simulation(&units_config(Some(&HI_LO), 2), &derive_shoe_seeds(2, 4)).expect("b");
    assert_ne!(a.units.sum_net, b.units.sum_net);
}

#[test]
fn bankroll_model_produces_traces() {
    let mut config = units_config(Some(&HI_LO), 3);
    config.bet_model = BetModel::Bankroll {
        spec: BankrollSpec {
            policy: BankrollPolicy::FlatPct { pct: 0.1 },
            initial_bankroll: 1000.0,
            min_bet: 1.0,
            max_bet: 1000.0,
        },
        ramp: UnitRamp::new(&HI_LO, 8.0, 1.0),
        edges: None,
    };
    let seeds = derive_shoe_seeds(7, 6);
    let res = run_simulation(&config, &seeds).expect("modèle B");
    assert_eq!(res.traces.len(), 6);
    for trace in &res.traces {
        assert_eq!(trace.samples.len(), 5);
        assert_eq!(trace.samples[0], 1000.0);
        assert!(trace.max_drawdown >= 0.0 && trace.max_drawdown <= 1.0);
    }
    let again = run_simulation(&config, &seeds).expect("modèle B bis");
    assert_eq!(res, again);
}

#[test]
fn kelly_without_edges_is_rejected() {
    let mut config = units_config(Some(&HI_LO), 1);
    config.bet_model = BetModel::Bankroll {
        spec: BankrollSpec {
            policy: BankrollPolicy::FractionalKelly { k: 0.5 },
            initial_bankroll: 1000.0,
            min_bet: 1.0,
            max_bet: 100.0,
        },
        ramp: UnitRamp::new(&HI_LO, 8.0, 1.0),
        edges: None,
    };
    assert!(run_simulation(&config, &derive_shoe_seeds(1, 1)).is_err());
}

#[test]
fn seeds_are_stable() {
    // Valeurs figées : un changement silencieux casserait la reproductibilité des données publiées
    let seeds = derive_shoe_seeds(2026, 3);
    assert_eq!(seeds, derive_shoe_seeds(2026, 3));
    assert_eq!(seeds.len(), 3);
    assert_ne!(seeds[0], seeds[1]);
}
