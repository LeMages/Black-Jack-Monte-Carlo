//! Orchestration des simulations : baseline, puis (système × config) en modèle A
//! et en modèle B, avec les mêmes seeds partout (CRN).

// Importation des modules standard
use std::collections::HashMap;
use std::time::Instant;

// Importation des crates externes
use anyhow::{Context, Result};

// Importation des modules internes
use crate::config::{PublishedConfig, RunConfig};
use engine::aggregate::{summarize_equity, EquityPoint, TcRow, UnitAccumulator, UnitMetrics};
use engine::betting::{BankrollPolicy, UnitRamp};
use engine::counters::CountSystem;
use engine::deviations::DeviationLevel;
use engine::simulation::{
    derive_shoe_seeds, run_simulation, BetModel, RunResults, SimulationConfig,
};

/// Résultats d'un couple (système × config).
#[derive(Debug, Clone)]
pub struct CellResult {
    /// Système simulé.
    pub system: &'static CountSystem,
    /// Identifiant de la config.
    pub config_id: String,
    /// Niveau de déviations.
    pub deviations: DeviationLevel,
    /// Métriques du modèle A.
    pub metrics: UnitMetrics,
    /// Distribution par bin de compte (modèle A).
    pub tc_rows: Vec<TcRow>,
    /// Courbe d'équité (modèle B).
    pub equity: Vec<EquityPoint>,
    /// Pire repli de chaque run (modèle B), en %.
    pub drawdowns_pct: Vec<f64>,
}

/// Ensemble des résultats d'un lancement.
#[derive(Debug, Clone)]
pub struct PlanResults {
    /// Basic strategy seule, mise plate.
    pub baseline: UnitMetrics,
    /// Résultats par (système × config), dans l'ordre configs puis systèmes.
    pub cells: Vec<CellResult>,
    /// Mains simulées au total (toutes passes confondues).
    pub total_hands: u64,
    /// Durée totale des simulations (s).
    pub elapsed_s: f64,
}

/// Clé du cache du modèle A : deux configs au même jeu partagent la passe A.
type UnitKey = (String, u64, u64, DeviationLevel, u64);

/// Exécute toutes les simulations décrites par la configuration.
///
/// # Arguments
///
/// * `config` - Configuration validée.
///
/// # Returns
///
/// Les résultats agrégés, ou la première erreur de simulation.
pub fn execute(config: &RunConfig) -> Result<PlanResults> {
    let started = Instant::now();
    let seeds = derive_shoe_seeds(config.master_seed, config.n_runs);
    let systems = config.resolved_systems()?;
    let mut total_hands = 0u64;

    // ------------------------------------------------------------------
    // 1. Ligne de référence : basic strategy seule, mise plate
    // ------------------------------------------------------------------
    let baseline_cfg = base_sim(
        config,
        config.rules,
        None,
        DeviationLevel::None,
        BetModel::Units(UnitRamp::flat()),
    );
    let baseline_res = timed_run("baseline", "basic strategy", &baseline_cfg, &seeds)?;
    total_hands += baseline_res.units.hands;
    let baseline = baseline_res
        .units
        .metrics(config.ror_bankroll_units)
        .map_err(anyhow::Error::msg)?;

    // ------------------------------------------------------------------
    // 2. Chaque config : modèle A (mis en cache) puis modèle B
    // ------------------------------------------------------------------
    let mut unit_cache: HashMap<UnitKey, UnitAccumulator> = HashMap::new();
    let mut cells = Vec::new();
    for published in &config.configs {
        for &system in &systems {
            let units = unit_pass(
                config,
                published,
                system,
                &seeds,
                &mut unit_cache,
                &mut total_hands,
            )?;
            let (equity, drawdowns_pct, hands) =
                bankroll_pass(config, published, system, &units, &seeds)?;
            total_hands += hands;
            cells.push(CellResult {
                system,
                config_id: published.id.clone(),
                deviations: published.deviations,
                metrics: units
                    .metrics(config.ror_bankroll_units)
                    .map_err(anyhow::Error::msg)?,
                tc_rows: units.tc_rows(config.tc_min_frequency),
                equity,
                drawdowns_pct,
            });
        }
    }
    Ok(PlanResults {
        baseline,
        cells,
        total_hands,
        elapsed_s: started.elapsed().as_secs_f64(),
    })
}

fn base_sim(
    config: &RunConfig,
    rules: engine::rules::Rules,
    system: Option<&'static CountSystem>,
    deviations: DeviationLevel,
    bet_model: BetModel,
) -> SimulationConfig {
    SimulationConfig {
        rules,
        system,
        deviations,
        bet_model,
        n_hands_per_run: config.n_hands_per_run,
        equity_sample_interval: config.equity_sample_interval,
        ace_factor: config.ace_factor,
        n_threads: config.n_threads,
    }
}

/// Passe du modèle A, réutilisée si une config précédente a le même jeu.
fn unit_pass(
    config: &RunConfig,
    published: &PublishedConfig,
    system: &'static CountSystem,
    seeds: &[u64],
    cache: &mut HashMap<UnitKey, UnitAccumulator>,
    total_hands: &mut u64,
) -> Result<UnitAccumulator> {
    let key: UnitKey = (
        system.id.to_string(),
        published.spread.to_bits(),
        published.ramp_offset.to_bits(),
        published.deviations,
        published.penetration.to_bits(),
    );
    if let Some(units) = cache.get(&key) {
        eprintln!(
            "[{}] {} modèle A : réutilisé (même jeu qu'une config précédente)",
            published.id, system.id
        );
        return Ok(units.clone());
    }
    let ramp = UnitRamp::new(system, published.spread, published.ramp_offset);
    let sim = base_sim(
        config,
        config.rules_for(published),
        Some(system),
        published.deviations,
        BetModel::Units(ramp),
    );
    let res = timed_run(
        &published.id,
        &format!("{} modèle A", system.id),
        &sim,
        seeds,
    )?;
    *total_hands += res.units.hands;
    cache.insert(key, res.units.clone());
    Ok(res.units)
}

/// Passe du modèle B : courbes d'équité et drawdowns.
fn bankroll_pass(
    config: &RunConfig,
    published: &PublishedConfig,
    system: &'static CountSystem,
    units: &UnitAccumulator,
    seeds: &[u64],
) -> Result<(Vec<EquityPoint>, Vec<f64>, u64)> {
    let spec = published.bankroll_policy.to_spec();
    let edges = match spec.policy {
        BankrollPolicy::FractionalKelly { .. } => Some(
            units
                .edge_table(config.kelly_min_hands)
                .map_err(anyhow::Error::msg)
                .with_context(|| format!("table Kelly de {} / {}", system.id, published.id))?,
        ),
        _ => None,
    };
    let bet_model = BetModel::Bankroll {
        spec,
        ramp: UnitRamp::new(system, published.spread, published.ramp_offset),
        edges,
    };
    let (hands, interval) = published.bankroll_horizon(config);
    let sim = SimulationConfig {
        n_hands_per_run: hands,
        equity_sample_interval: interval,
        ..base_sim(
            config,
            config.rules_for(published),
            Some(system),
            published.deviations,
            bet_model,
        )
    };
    let res = timed_run(
        &published.id,
        &format!("{} modèle B", system.id),
        &sim,
        seeds,
    )?;
    let equity = summarize_equity(&res.traces, interval).map_err(anyhow::Error::msg)?;
    let drawdowns = res.traces.iter().map(|t| 100.0 * t.max_drawdown).collect();
    Ok((equity, drawdowns, res.units.hands))
}

/// Lance une simulation et journalise le débit.
fn timed_run(scope: &str, what: &str, sim: &SimulationConfig, seeds: &[u64]) -> Result<RunResults> {
    let start = Instant::now();
    let res = run_simulation(sim, seeds)
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("simulation {scope} / {what}"))?;
    let secs = start.elapsed().as_secs_f64().max(1e-9);
    eprintln!(
        "[{scope}] {what} : {} mains en {secs:.1} s ({:.1} M mains/s)",
        res.units.hands,
        res.units.hands as f64 / secs / 1e6
    );
    Ok(res)
}
