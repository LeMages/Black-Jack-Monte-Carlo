//! Orchestration des runs Monte Carlo (CRN, rayon, réduction ordonnée).
//!
//! * Les seeds de run sont dérivées une seule fois de `master_seed` par
//!   [`derive_shoe_seeds`] et rejouées à l'identique pour chaque (système × config) :
//!   ce sont les *Common Random Numbers*.
//! * Dans un run, le k-ième sabot est mélangé avec un PRNG Xoshiro256++ semé par
//!   `(seed du run, k)` : le k-ième sabot est donc le même pour tous les systèmes,
//!   seul l'instant du remélange (fin de main) peut différer.
//! * Les runs sont répartis par `rayon` dans un pool local de `n_threads`, collectés
//!   dans l'ordre des seeds puis réduits séquentiellement : même seed, mêmes
//!   résultats au bit près, quel que soit le nombre de threads.

// Importation des crates externes
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use rayon::prelude::*;

// Importation des modules internes
use crate::aggregate::{BankrollTrace, UnitAccumulator};
use crate::betting::{tc_bin, BankrollSpec, BankrollState, EdgeTable, UnitRamp};
use crate::cards::Shoe;
use crate::counters::{CountSystem, Counter, NullCounter, TagCounter};
use crate::deviations::{DeviationLevel, DeviationStrategy};
use crate::game::play_round;
use crate::rules::Rules;
use crate::strategy::{BasicStrategy, PlayingStrategy};

/// Modèle de pari d'une simulation.
#[derive(Debug, Clone, PartialEq)]
pub enum BetModel {
    /// Modèle A : unités, bankroll infinie.
    Units(UnitRamp),
    /// Modèle B : bankroll finie avec politique de mise.
    Bankroll {
        /// Paramètres de bankroll.
        spec: BankrollSpec,
        /// Rampe (politique proportionnelle au compte).
        ramp: UnitRamp,
        /// Avantage par bin (politique Kelly).
        edges: Option<EdgeTable>,
    },
}

/// Paramètres d'une simulation (un système, une config, un modèle de pari).
///
/// Attributs principaux :
/// * `system` - système de comptage, `None` pour la basic strategy seule ;
/// * `deviations` - niveau de déviations ;
/// * `bet_model` - modèle A ou B ;
/// * `n_hands_per_run` - mains par run (une seed = un run).
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationConfig {
    /// Règles de la table.
    pub rules: Rules,
    /// Système de comptage, `None` pour la basic strategy seule.
    pub system: Option<&'static CountSystem>,
    /// Niveau de déviations (ignoré sans système).
    pub deviations: DeviationLevel,
    /// Modèle de pari.
    pub bet_model: BetModel,
    /// Mains par run.
    pub n_hands_per_run: u64,
    /// Mains entre deux points de la courbe d'équité (modèle B).
    pub equity_sample_interval: u64,
    /// Poids du side-count des As.
    pub ace_factor: f64,
    /// Nombre de threads du pool rayon.
    pub n_threads: usize,
}

/// Résultats d'un run.
#[derive(Debug, Clone, PartialEq)]
pub struct RunStats {
    /// Sommes des mains jouées.
    pub units: UnitAccumulator,
    /// Trajectoire de bankroll (modèle B uniquement).
    pub trace: Option<BankrollTrace>,
}

/// Résultats agrégés d'une simulation.
///
/// Attributs principaux :
/// * `units` - accumulateur fusionné de tous les runs ;
/// * `traces` - trajectoires de bankroll dans l'ordre des seeds (modèle B).
#[derive(Debug, Clone, PartialEq)]
pub struct RunResults {
    /// Accumulateur fusionné.
    pub units: UnitAccumulator,
    /// Trajectoires de bankroll (vide en modèle A).
    pub traces: Vec<BankrollTrace>,
}

/// Mélangeur SplitMix64 : diffuse un entier 64 bits.
///
/// # Arguments
///
/// * `x` - Valeur d'entrée.
///
/// # Returns
///
/// La valeur mélangée.
pub fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Dérive les seeds de run à partir de la seed maîtresse.
///
/// # Arguments
///
/// * `master_seed` - Seed maîtresse de la configuration.
/// * `n_runs` - Nombre de runs.
///
/// # Returns
///
/// Une seed par run, identique pour tous les systèmes (CRN).
pub fn derive_shoe_seeds(master_seed: u64, n_runs: usize) -> Vec<u64> {
    (0..n_runs as u64)
        .map(|i| splitmix64(master_seed ^ splitmix64(i)))
        .collect()
}

/// Lance une simulation sur les seeds fournies.
///
/// # Arguments
///
/// * `config` - Paramètres de simulation.
/// * `shoe_seeds` - Seeds de run (CRN : les mêmes pour chaque système).
///
/// # Returns
///
/// Les résultats agrégés, ou une erreur de configuration.
pub fn run_simulation(config: &SimulationConfig, shoe_seeds: &[u64]) -> Result<RunResults, String> {
    validate(config, shoe_seeds)?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.n_threads)
        .build()
        .map_err(|e| {
            format!(
                "création du pool rayon ({} threads) : {e}",
                config.n_threads
            )
        })?;
    // collect() conserve l'ordre des seeds : la réduction qui suit est ordonnée
    let runs: Vec<RunStats> = pool.install(|| {
        shoe_seeds
            .par_iter()
            .map(|&seed| dispatch_run(config, seed))
            .collect()
    });
    let mut units = UnitAccumulator::default();
    let mut traces = Vec::new();
    for run in runs {
        units.merge(&run.units);
        if let Some(trace) = run.trace {
            traces.push(trace);
        }
    }
    Ok(RunResults { units, traces })
}

fn validate(config: &SimulationConfig, shoe_seeds: &[u64]) -> Result<(), String> {
    config.rules.validate()?;
    if shoe_seeds.is_empty() {
        return Err("aucune seed de run".to_string());
    }
    if config.n_hands_per_run == 0 || config.n_threads == 0 {
        return Err("n_hands_per_run et n_threads doivent être > 0".to_string());
    }
    if let BetModel::Bankroll { spec, edges, .. } = &config.bet_model {
        spec.validate()?;
        if config.equity_sample_interval == 0 {
            return Err("equity_sample_interval doit être > 0".to_string());
        }
        let kelly = matches!(
            spec.policy,
            crate::betting::BankrollPolicy::FractionalKelly { .. }
        );
        if kelly && edges.is_none() {
            return Err("politique Kelly sans table d'avantage".to_string());
        }
    }
    Ok(())
}

/// Choisit compteur et stratégie concrets, puis lance le run monomorphisé.
fn dispatch_run(config: &SimulationConfig, seed: u64) -> RunStats {
    let basic = BasicStrategy::new(&config.rules);
    match config.system {
        None => simulate_run(config, &basic, NullCounter, seed),
        Some(system) => {
            let strategy = DeviationStrategy::new(basic, config.deviations, system);
            let counter = TagCounter::new(system, config.rules.n_decks, config.ace_factor);
            simulate_run(config, &strategy, counter, seed)
        }
    }
}

/// Mélange le k-ième sabot d'un run avec son propre PRNG.
fn shuffle_shoe(shoe: &mut Shoe, run_seed: u64, shoe_index: u64) {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(splitmix64(run_seed ^ splitmix64(!shoe_index)));
    shoe.shuffle(&mut rng);
}

/// Joue un run complet.
fn simulate_run<S: PlayingStrategy, C: Counter>(
    config: &SimulationConfig,
    strategy: &S,
    mut counter: C,
    seed: u64,
) -> RunStats {
    let rules = &config.rules;
    let mut shoe = Shoe::new(rules.n_decks, rules.penetration);
    let mut shoe_index = 0;
    shuffle_shoe(&mut shoe, seed, shoe_index);
    counter.reset(rules.n_decks);
    let mut units = UnitAccumulator::default();
    let mut bankroll = BankrollRun::new(config);

    for hand in 0..config.n_hands_per_run {
        // Remélange uniquement entre deux mains, compteur remis à l'IRC
        if shoe.needs_reshuffle() {
            shoe_index += 1;
            shuffle_shoe(&mut shoe, seed, shoe_index);
            counter.reset(rules.n_decks);
        }
        let betting_count = counter.betting_count(shoe.decks_remaining());
        let bet = match (&config.bet_model, bankroll.as_ref()) {
            (BetModel::Units(ramp), _) => ramp.units(betting_count),
            (BetModel::Bankroll { spec, ramp, edges }, Some(run)) => {
                match run
                    .state
                    .next_bet(spec, ramp, edges.as_ref(), betting_count)
                {
                    Some(bet) => bet,
                    None => break,
                }
            }
            (BetModel::Bankroll { .. }, None) => unreachable!("modèle B sans état de bankroll"),
        };
        let out = play_round(&mut shoe, rules, strategy, &mut counter, bet);
        units.record(tc_bin(betting_count), &out);
        if let Some(run) = bankroll.as_mut() {
            run.settle(out.net, hand + 1);
        }
    }
    RunStats {
        units,
        trace: bankroll.map(BankrollRun::finish),
    }
}

/// Suivi d'une bankroll du modèle B pendant un run.
struct BankrollRun {
    state: BankrollState,
    min_bet: f64,
    interval: u64,
    n_samples: usize,
    samples: Vec<f64>,
    ruined: Vec<bool>,
}

impl BankrollRun {
    fn new(config: &SimulationConfig) -> Option<Self> {
        let BetModel::Bankroll { spec, .. } = &config.bet_model else {
            return None;
        };
        let interval = config.equity_sample_interval;
        let n_samples = (config.n_hands_per_run / interval) as usize + 1;
        let mut samples = Vec::with_capacity(n_samples);
        samples.push(spec.initial_bankroll);
        Some(Self {
            state: BankrollState::new(spec.initial_bankroll),
            min_bet: spec.min_bet,
            interval,
            n_samples,
            samples,
            ruined: vec![false],
        })
    }

    fn settle(&mut self, net: f64, hands_played: u64) {
        self.state.settle(net, self.min_bet);
        if hands_played.is_multiple_of(self.interval) {
            self.samples.push(self.state.bankroll);
            self.ruined.push(self.state.ruined);
        }
    }

    /// Complète les points restants avec la bankroll figée d'un run ruiné.
    fn finish(mut self) -> BankrollTrace {
        while self.samples.len() < self.n_samples {
            self.samples.push(self.state.bankroll);
            self.ruined.push(self.state.ruined);
        }
        BankrollTrace {
            samples: self.samples,
            ruined: self.ruined,
            max_drawdown: self.state.max_drawdown,
        }
    }
}

#[cfg(test)]
mod tests;
