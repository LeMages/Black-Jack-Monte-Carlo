//! Tests des modèles de pari.

use super::*;
use crate::counters::{HI_LO, KO, ZEN};

#[test]
fn hi_lo_ramp_follows_formula() {
    let ramp = UnitRamp::new(&HI_LO, 15.0, 1.0);
    assert_eq!(ramp.units(-3.0), 1.0);
    assert_eq!(ramp.units(1.9), 1.0);
    assert_eq!(ramp.units(3.0), 2.0);
    assert_eq!(ramp.units(5.5), 4.0);
    assert_eq!(ramp.units(40.0), 15.0);
}

#[test]
fn level_two_ramp_is_rescaled_to_hi_lo() {
    let ramp = UnitRamp::new(&ZEN, 15.0, 1.0);
    let scale = ZEN.scale_vs_hi_lo();
    assert_eq!(ramp.units(3.0 * scale), 2.0);
}

#[test]
fn ko_ramp_starts_at_pivot() {
    let ramp = UnitRamp::new(&KO, 15.0, 1.0);
    assert_eq!(ramp.units(3.0), 1.0);
    assert_eq!(ramp.units(4.0), 2.0);
    assert_eq!(ramp.units(7.0), 5.0);
    assert_eq!(ramp.units(-20.0), 1.0);
}

#[test]
fn risk_metrics() {
    assert_eq!(n0(0.02, 2.0), 10_000.0);
    assert_eq!(n0(-0.01, 1.0), f64::INFINITY);
    assert!((risk_of_ruin(0.02, 2.0, 100.0) - (-1.0f64).exp()).abs() < 1e-12);
    assert_eq!(risk_of_ruin(-0.01, 1.0, 100.0), 1.0);
    assert!((kelly_fraction(0.02, 2.0) - 0.005).abs() < 1e-12);
    assert_eq!(kelly_fraction(-0.02, 2.0), 0.0);
}

fn spec(policy: BankrollPolicy) -> BankrollSpec {
    BankrollSpec {
        policy,
        initial_bankroll: 1000.0,
        min_bet: 1.0,
        max_bet: 100.0,
    }
}

#[test]
fn flat_pct_bets_fraction_of_bankroll() {
    let s = spec(BankrollPolicy::FlatPct { pct: 0.1 });
    let state = BankrollState::new(1000.0);
    let ramp = UnitRamp::flat();
    assert_eq!(state.next_bet(&s, &ramp, None, 0.0), Some(100.0));
    let small = BankrollState::new(5.0);
    assert_eq!(small.next_bet(&s, &ramp, None, 0.0), Some(1.0));
}

#[test]
fn kelly_uses_edge_table_and_caps() {
    let bins = [
        (0, 1000, -5.0, 1000.0),
        (2, 1000, 10.0, 1000.0),
        (5, 10, 9.0, 10.0),
    ];
    let table = EdgeTable::from_bins(&bins, 1.25, 100).expect("table valide");
    assert_eq!(table.edge(0.4), -0.005);
    assert_eq!(table.edge(2.0), 0.01);
    // Le bin 5 est trop peu observé : il reprend le bin fiable le plus proche
    assert_eq!(table.edge(5.0), 0.01);
    let s = spec(BankrollPolicy::FractionalKelly { k: 0.5 });
    let ramp = UnitRamp::flat();
    let state = BankrollState::new(1000.0);
    assert_eq!(state.next_bet(&s, &ramp, Some(&table), 0.0), Some(1.0));
    let bet = state.next_bet(&s, &ramp, Some(&table), 2.0).expect("mise");
    assert!((bet - 4.0).abs() < 1e-12);
    let rich = BankrollState::new(1e6);
    assert_eq!(rich.next_bet(&s, &ramp, Some(&table), 2.0), Some(100.0));
}

#[test]
fn ruin_freezes_and_drawdown_tracks_peak() {
    let mut state = BankrollState::new(100.0);
    state.settle(100.0, 1.0);
    state.settle(-50.0, 1.0);
    assert!((state.max_drawdown - 0.25).abs() < 1e-12);
    state.settle(-149.5, 1.0);
    assert!(state.ruined);
    assert_eq!(
        state.next_bet(
            &spec(BankrollPolicy::FlatPct { pct: 0.1 }),
            &UnitRamp::flat(),
            None,
            0.0
        ),
        None
    );
}

#[test]
fn spec_validation() {
    assert!(spec(BankrollPolicy::FlatPct { pct: 0.1 })
        .validate()
        .is_ok());
    assert!(spec(BankrollPolicy::FlatPct { pct: 1.5 })
        .validate()
        .is_err());
}
