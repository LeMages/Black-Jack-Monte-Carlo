//! Conversion des résultats en fichiers du contrat de données.
//!
//! Tables (Parquet + JSON) : `metrics`, `equity_curves`, `tc_distribution`,
//! `drawdowns`, plus `drawdown_histogram` (binning fait ici, pas dans le front).
//! Documents JSON : `configs`, `baseline`, `systems`, `meta`.

// Importation des crates externes
use anyhow::Result;
use serde::Serialize;

// Importation des modules internes
use crate::config::RunConfig;
use crate::run_plan::{CellResult, PlanResults};
use crate::table::{Column, Table};
use engine::aggregate::{histogram_pct, UnitMetrics};
use engine::counters::{CountSystem, KO_PIVOT};
use engine::deviations::DeviationLevel;
use engine::rules::Rules;

/// Libellé d'export d'un niveau de déviations (même valeur que dans le TOML).
pub fn deviation_label(level: DeviationLevel) -> &'static str {
    match level {
        DeviationLevel::None => "none",
        DeviationLevel::Insurance => "insurance",
        DeviationLevel::I18 => "i18",
        DeviationLevel::I18Fab4 => "i18_fab4",
    }
}

fn strings<F: Fn(&CellResult) -> String>(cells: &[CellResult], f: F) -> Column {
    Column::Str(cells.iter().map(f).collect())
}

fn floats<F: Fn(&UnitMetrics) -> f64>(cells: &[CellResult], f: F) -> Column {
    Column::F64(cells.iter().map(|c| f(&c.metrics)).collect())
}

/// Table `metrics` : une ligne par (système × config).
pub fn metrics_table(cells: &[CellResult]) -> Result<Table> {
    Table::new(
        "metrics",
        vec![
            ("system", strings(cells, |c| c.system.id.to_string())),
            ("config_id", strings(cells, |c| c.config_id.clone())),
            ("bet_model", strings(cells, |_| "units".to_string())),
            (
                "deviations",
                strings(cells, |c| deviation_label(c.deviations).to_string()),
            ),
            ("advantage_pct", floats(cells, |m| m.advantage_pct)),
            ("ev_per_100", floats(cells, |m| m.ev_per_100)),
            ("std_per_hand", floats(cells, |m| m.std_per_hand)),
            (
                "n0",
                Column::OptF64(cells.iter().map(|c| c.metrics.n0).collect()),
            ),
            ("risk_of_ruin", floats(cells, |m| m.risk_of_ruin)),
            ("kelly_fraction", floats(cells, |m| m.kelly_fraction)),
            ("avg_bet", floats(cells, |m| m.avg_bet)),
            ("win_rate", floats(cells, |m| m.win_rate)),
            (
                "n_hands_simulated",
                Column::U64(cells.iter().map(|c| c.metrics.n_hands_simulated).collect()),
            ),
        ],
    )
}

/// Table `equity_curves` (format long).
pub fn equity_table(cells: &[CellResult]) -> Result<Table> {
    let rows: Vec<(&CellResult, &engine::aggregate::EquityPoint)> = cells
        .iter()
        .flat_map(|c| c.equity.iter().map(move |p| (c, p)))
        .collect();
    let col = |f: fn(&engine::aggregate::EquityPoint) -> f64| {
        Column::F64(rows.iter().map(|(_, p)| f(p)).collect())
    };
    Table::new(
        "equity_curves",
        vec![
            (
                "system",
                Column::Str(rows.iter().map(|(c, _)| c.system.id.to_string()).collect()),
            ),
            (
                "config_id",
                Column::Str(rows.iter().map(|(c, _)| c.config_id.clone()).collect()),
            ),
            (
                "hand_index",
                Column::U64(rows.iter().map(|(_, p)| p.hand_index).collect()),
            ),
            ("p05", col(|p| p.p05)),
            ("p25", col(|p| p.p25)),
            ("p50", col(|p| p.p50)),
            ("p75", col(|p| p.p75)),
            ("p95", col(|p| p.p95)),
            ("mean", col(|p| p.mean)),
            ("frac_ruined", col(|p| p.frac_ruined)),
        ],
    )
}

/// Table `tc_distribution`.
pub fn tc_table(cells: &[CellResult]) -> Result<Table> {
    let rows: Vec<(&CellResult, &engine::aggregate::TcRow)> = cells
        .iter()
        .flat_map(|c| c.tc_rows.iter().map(move |r| (c, r)))
        .collect();
    Table::new(
        "tc_distribution",
        vec![
            (
                "system",
                Column::Str(rows.iter().map(|(c, _)| c.system.id.to_string()).collect()),
            ),
            (
                "config_id",
                Column::Str(rows.iter().map(|(c, _)| c.config_id.clone()).collect()),
            ),
            (
                "true_count",
                Column::I32(rows.iter().map(|(_, r)| r.true_count).collect()),
            ),
            (
                "frequency",
                Column::F64(rows.iter().map(|(_, r)| r.frequency).collect()),
            ),
            (
                "advantage_at_tc",
                Column::F64(rows.iter().map(|(_, r)| r.advantage_at_tc).collect()),
            ),
            (
                "avg_bet_at_tc",
                Column::F64(rows.iter().map(|(_, r)| r.avg_bet_at_tc).collect()),
            ),
        ],
    )
}

/// Table `drawdowns` : une valeur par run.
pub fn drawdowns_table(cells: &[CellResult]) -> Result<Table> {
    let rows: Vec<(&CellResult, f64)> = cells
        .iter()
        .flat_map(|c| c.drawdowns_pct.iter().map(move |&d| (c, d)))
        .collect();
    Table::new(
        "drawdowns",
        vec![
            (
                "system",
                Column::Str(rows.iter().map(|(c, _)| c.system.id.to_string()).collect()),
            ),
            (
                "config_id",
                Column::Str(rows.iter().map(|(c, _)| c.config_id.clone()).collect()),
            ),
            (
                "max_drawdown_pct",
                Column::F64(rows.iter().map(|&(_, d)| d).collect()),
            ),
        ],
    )
}

/// Table `drawdown_histogram` : histogramme des drawdowns, binné à l'export.
pub fn drawdown_histogram_table(cells: &[CellResult], bin_width: f64) -> Result<Table> {
    let rows: Vec<(&CellResult, engine::aggregate::HistogramBin)> = cells
        .iter()
        .flat_map(|c| {
            histogram_pct(&c.drawdowns_pct, bin_width)
                .into_iter()
                .map(move |b| (c, b))
        })
        .collect();
    Table::new(
        "drawdown_histogram",
        vec![
            (
                "system",
                Column::Str(rows.iter().map(|(c, _)| c.system.id.to_string()).collect()),
            ),
            (
                "config_id",
                Column::Str(rows.iter().map(|(c, _)| c.config_id.clone()).collect()),
            ),
            (
                "bin_start",
                Column::F64(rows.iter().map(|(_, b)| b.bin_start).collect()),
            ),
            (
                "bin_end",
                Column::F64(rows.iter().map(|(_, b)| b.bin_end).collect()),
            ),
            (
                "count",
                Column::U64(rows.iter().map(|(_, b)| b.count).collect()),
            ),
        ],
    )
}

/// Entrée de `configs.json`.
#[derive(Debug, Serialize)]
pub struct ConfigEntry {
    id: String,
    label: String,
    bankroll_policy: String,
    spread: f64,
    ramp_offset: f64,
    deviations: &'static str,
    penetration: f64,
    bankroll_hands: u64,
    bankroll_sample_interval: u64,
    initial_bankroll: f64,
    min_bet: f64,
    max_bet: f64,
}

/// Contenu de `configs.json`.
pub fn configs_doc(config: &RunConfig) -> Vec<ConfigEntry> {
    config
        .configs
        .iter()
        .map(|c| ConfigEntry {
            id: c.id.clone(),
            label: c.label.clone(),
            bankroll_policy: c.bankroll_policy.describe(),
            spread: c.spread,
            ramp_offset: c.ramp_offset,
            deviations: deviation_label(c.deviations),
            penetration: c.penetration,
            bankroll_hands: c.bankroll_horizon(config).0,
            bankroll_sample_interval: c.bankroll_horizon(config).1,
            initial_bankroll: c.bankroll_policy.initial_bankroll,
            min_bet: c.bankroll_policy.min_bet,
            max_bet: c.bankroll_policy.max_bet,
        })
        .collect()
}

/// Contenu de `baseline.json`.
#[derive(Debug, Serialize)]
pub struct BaselineDoc {
    system: &'static str,
    config_id: &'static str,
    bet_model: &'static str,
    deviations: &'static str,
    #[serde(flatten)]
    metrics: UnitMetrics,
}

/// Construit `baseline.json`.
pub fn baseline_doc(results: &PlanResults) -> BaselineDoc {
    BaselineDoc {
        system: "basic",
        config_id: "baseline",
        bet_model: "flat",
        deviations: "none",
        metrics: results.baseline,
    }
}

/// Entrée de `systems.json` : définition exacte utilisée par le moteur.
#[derive(Debug, Serialize)]
pub struct SystemDoc {
    #[serde(flatten)]
    system: CountSystem,
    initial_running_count: f64,
    pivot: Option<f64>,
    index_scale: f64,
    ace_factor: Option<f64>,
}

/// Construit `systems.json`.
pub fn systems_doc(config: &RunConfig) -> Result<Vec<SystemDoc>> {
    Ok(config
        .resolved_systems()?
        .into_iter()
        .map(|s| SystemDoc {
            system: *s,
            initial_running_count: s.initial_running_count(config.rules.n_decks),
            pivot: (!s.balanced).then_some(KO_PIVOT),
            index_scale: s.scale_vs_hi_lo(),
            ace_factor: s.ace_side_count.then_some(config.ace_factor),
        })
        .collect())
}

/// Contenu de `meta.json` : paramètres du lancement.
#[derive(Debug, Serialize)]
pub struct MetaDoc {
    master_seed: u64,
    n_runs: usize,
    n_hands_per_run: u64,
    n_threads: usize,
    equity_sample_interval: u64,
    ror_bankroll_units: f64,
    total_hands_simulated: u64,
    elapsed_s: f64,
    rules: Rules,
}

/// Construit `meta.json`.
pub fn meta_doc(config: &RunConfig, results: &PlanResults) -> MetaDoc {
    MetaDoc {
        master_seed: config.master_seed,
        n_runs: config.n_runs,
        n_hands_per_run: config.n_hands_per_run,
        n_threads: config.n_threads,
        equity_sample_interval: config.equity_sample_interval,
        ror_bankroll_units: config.ror_bankroll_units,
        total_hands_simulated: results.total_hands,
        elapsed_s: results.elapsed_s,
        rules: config.rules,
    }
}
