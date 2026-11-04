//! Tests de l'agrégation.

use super::*;

fn outcome(net: f64, bet: f64) -> RoundOutcome {
    RoundOutcome {
        net,
        initial_bet: bet,
        total_wagered: bet,
    }
}

#[test]
fn metrics_from_known_outcomes() {
    let mut acc = UnitAccumulator::default();
    acc.record(0, &outcome(1.0, 1.0));
    acc.record(0, &outcome(-1.0, 1.0));
    acc.record(2, &outcome(4.0, 2.0));
    acc.record(2, &outcome(0.0, 2.0));
    let m = acc.metrics(1000.0).expect("métriques");
    assert_eq!(m.n_hands_simulated, 4);
    assert!((m.advantage_pct - 100.0 * 4.0 / 6.0).abs() < 1e-12);
    assert!((m.ev_per_100 - 100.0).abs() < 1e-12);
    assert!((m.avg_bet - 1.5).abs() < 1e-12);
    assert_eq!(m.win_rate, 0.5);
    let rows = acc.tc_rows(0.0);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].true_count, 2);
    assert!((rows[1].advantage_at_tc - 100.0).abs() < 1e-12);
    assert!((rows[1].avg_bet_at_tc - 2.0).abs() < 1e-12);
}

#[test]
fn merge_equals_single_accumulation() {
    let data = [
        (0, 1.0, 1.0),
        (1, -2.0, 2.0),
        (-3, 1.5, 1.0),
        (5, -0.5, 1.0),
    ];
    let mut whole = UnitAccumulator::default();
    for &(b, n, w) in &data {
        whole.record(b, &outcome(n, w));
    }
    let mut a = UnitAccumulator::default();
    let mut b = UnitAccumulator::default();
    for &(bin, n, w) in &data[..2] {
        a.record(bin, &outcome(n, w));
    }
    for &(bin, n, w) in &data[2..] {
        b.record(bin, &outcome(n, w));
    }
    a.merge(&b);
    assert_eq!(a, whole);
}

#[test]
fn negative_ev_has_no_n0() {
    let mut acc = UnitAccumulator::default();
    acc.record(0, &outcome(-1.0, 1.0));
    acc.record(0, &outcome(0.5, 1.0));
    let m = acc.metrics(1000.0).expect("métriques");
    assert_eq!(m.n0, None);
    assert_eq!(m.risk_of_ruin, 1.0);
}

#[test]
fn percentiles_interpolate() {
    let v = [1.0, 2.0, 3.0, 4.0, 5.0];
    assert_eq!(percentile(&v, 0.5), 3.0);
    assert_eq!(percentile(&v, 0.0), 1.0);
    assert_eq!(percentile(&v, 1.0), 5.0);
    assert!((percentile(&v, 0.25) - 2.0).abs() < 1e-12);
    assert!((percentile(&[0.0, 10.0], 0.05) - 0.5).abs() < 1e-12);
}

#[test]
fn equity_summary_and_histogram() {
    let traces = vec![
        BankrollTrace {
            samples: vec![100.0, 50.0],
            ruined: vec![false, false],
            max_drawdown: 0.5,
        },
        BankrollTrace {
            samples: vec![100.0, 0.5],
            ruined: vec![false, true],
            max_drawdown: 0.995,
        },
    ];
    let points = summarize_equity(&traces, 10).expect("résumé");
    assert_eq!(points[1].hand_index, 10);
    assert_eq!(points[1].frac_ruined, 0.5);
    assert!((points[1].mean - 25.25).abs() < 1e-12);
    let hist = histogram_pct(&[0.0, 4.9, 5.0, 100.0], 5.0);
    assert_eq!(hist.len(), 20);
    assert_eq!(hist[0].count, 2);
    assert_eq!(hist[1].count, 1);
    assert_eq!(hist[19].count, 1);
}
