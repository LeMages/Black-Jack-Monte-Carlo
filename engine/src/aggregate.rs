//! Agrégation des résultats de simulation.
//!
//! Les accumulateurs sont remplis run par run puis fusionnés dans l'ordre des
//! seeds : la réduction est séquentielle et ordonnée, donc le résultat ne dépend
//! pas de l'ordonnancement des threads. Toutes les statistiques exportées
//! (moyennes, percentiles, histogrammes) sont calculées ici, jamais côté dashboard.

// Importation des crates externes
use serde::Serialize;

// Importation des modules internes
use crate::betting::{kelly_fraction, n0, risk_of_ruin, EdgeTable, MAX_TC_BIN, MIN_TC_BIN};
use crate::game::RoundOutcome;

/// Nombre de bins de compte suivis.
pub const N_TC_BINS: usize = (MAX_TC_BIN - MIN_TC_BIN + 1) as usize;

/// Sommes d'un bin de compte.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TcBinSums {
    /// Mains jouées dans ce bin.
    pub hands: u64,
    /// Gain net cumulé.
    pub net: f64,
    /// Mises initiales cumulées.
    pub bet: f64,
}

/// Accumulateur des mains d'un ou plusieurs runs.
///
/// Attributs principaux :
/// * `hands` - mains jouées ;
/// * `sum_net` / `sum_net_sq` - somme et somme des carrés du gain net ;
/// * `sum_bet` - somme des mises initiales ;
/// * `sum_unit` / `sum_unit_sq` - résultat par unité misée (net / mise) ;
/// * `tc_bins` - sommes par bin de compte de mise.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitAccumulator {
    /// Mains jouées.
    pub hands: u64,
    /// Mains gagnées (gain net strictement positif).
    pub wins: u64,
    /// Somme des gains nets.
    pub sum_net: f64,
    /// Somme des carrés des gains nets.
    pub sum_net_sq: f64,
    /// Somme des mises initiales.
    pub sum_bet: f64,
    /// Somme des gains par unité misée.
    pub sum_unit: f64,
    /// Somme des carrés des gains par unité misée.
    pub sum_unit_sq: f64,
    /// Sommes par bin de compte, de `MIN_TC_BIN` à `MAX_TC_BIN`.
    pub tc_bins: Vec<TcBinSums>,
}

impl Default for UnitAccumulator {
    fn default() -> Self {
        Self {
            hands: 0,
            wins: 0,
            sum_net: 0.0,
            sum_net_sq: 0.0,
            sum_bet: 0.0,
            sum_unit: 0.0,
            sum_unit_sq: 0.0,
            tc_bins: vec![TcBinSums::default(); N_TC_BINS],
        }
    }
}

/// Métriques du modèle A pour un (système × config).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct UnitMetrics {
    /// Gain total / mises initiales totales, en %.
    pub advantage_pct: f64,
    /// Espérance pour 100 mains (unités).
    pub ev_per_100: f64,
    /// Écart-type par main (unités).
    pub std_per_hand: f64,
    /// N₀ = (σ / EV)², `None` si l'espérance est ≤ 0.
    pub n0: Option<f64>,
    /// Risque de ruine pour la bankroll de référence.
    pub risk_of_ruin: f64,
    /// Taille d'unité Kelly en fraction de bankroll (EV / σ²).
    pub kelly_fraction: f64,
    /// Mise moyenne (unités).
    pub avg_bet: f64,
    /// Fraction des mains au gain net positif.
    pub win_rate: f64,
    /// Nombre de mains simulées.
    pub n_hands_simulated: u64,
}

/// Ligne de distribution par bin de compte.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TcRow {
    /// Bin de compte (entier, `floor`).
    pub true_count: i32,
    /// Fraction des mains jouées dans ce bin.
    pub frequency: f64,
    /// Avantage par unité misée dans ce bin, en %.
    pub advantage_at_tc: f64,
    /// Mise moyenne dans ce bin (unités).
    pub avg_bet_at_tc: f64,
}

impl UnitAccumulator {
    /// Enregistre une main.
    ///
    /// # Arguments
    ///
    /// * `bin` - Bin du compte de mise au moment de miser.
    /// * `out` - Résultat de la main.
    pub fn record(&mut self, bin: i32, out: &RoundOutcome) {
        let unit = out.net / out.initial_bet;
        self.hands += 1;
        self.wins += u64::from(out.net > 0.0);
        self.sum_net += out.net;
        self.sum_net_sq += out.net * out.net;
        self.sum_bet += out.initial_bet;
        self.sum_unit += unit;
        self.sum_unit_sq += unit * unit;
        let slot = &mut self.tc_bins[(bin - MIN_TC_BIN) as usize];
        slot.hands += 1;
        slot.net += out.net;
        slot.bet += out.initial_bet;
    }

    /// Fusionne un autre accumulateur (appelé dans l'ordre des seeds).
    ///
    /// # Arguments
    ///
    /// * `other` - Accumulateur d'un run suivant.
    pub fn merge(&mut self, other: &UnitAccumulator) {
        self.hands += other.hands;
        self.wins += other.wins;
        self.sum_net += other.sum_net;
        self.sum_net_sq += other.sum_net_sq;
        self.sum_bet += other.sum_bet;
        self.sum_unit += other.sum_unit;
        self.sum_unit_sq += other.sum_unit_sq;
        for (mine, theirs) in self.tc_bins.iter_mut().zip(&other.tc_bins) {
            mine.hands += theirs.hands;
            mine.net += theirs.net;
            mine.bet += theirs.bet;
        }
    }

    /// Calcule les métriques du modèle A.
    ///
    /// # Arguments
    ///
    /// * `ror_bankroll_units` - Bankroll de référence du risque de ruine.
    ///
    /// # Returns
    ///
    /// Les métriques, ou une erreur si aucune main n'a été jouée.
    pub fn metrics(&self, ror_bankroll_units: f64) -> Result<UnitMetrics, String> {
        if self.hands == 0 {
            return Err("aucune main simulée".to_string());
        }
        let n = self.hands as f64;
        let ev = self.sum_net / n;
        let std = (self.sum_net_sq / n - ev * ev).max(0.0).sqrt();
        let n0_value = n0(ev, std);
        Ok(UnitMetrics {
            advantage_pct: 100.0 * self.sum_net / self.sum_bet,
            ev_per_100: 100.0 * ev,
            std_per_hand: std,
            n0: n0_value.is_finite().then_some(n0_value),
            risk_of_ruin: risk_of_ruin(ev, std, ror_bankroll_units),
            kelly_fraction: kelly_fraction(ev, std),
            avg_bet: self.sum_bet / n,
            win_rate: self.wins as f64 / n,
            n_hands_simulated: self.hands,
        })
    }

    /// Variance du résultat par unité misée.
    pub fn unit_variance(&self) -> f64 {
        if self.hands == 0 {
            return 0.0;
        }
        let n = self.hands as f64;
        let mean = self.sum_unit / n;
        self.sum_unit_sq / n - mean * mean
    }

    /// Distribution par bin de compte, limitée aux bins assez fréquents.
    ///
    /// # Arguments
    ///
    /// * `min_frequency` - Fréquence minimale d'un bin exporté.
    ///
    /// # Returns
    ///
    /// Les lignes triées par bin croissant.
    pub fn tc_rows(&self, min_frequency: f64) -> Vec<TcRow> {
        let total = self.hands.max(1) as f64;
        self.tc_bins
            .iter()
            .enumerate()
            .filter(|(_, b)| b.hands > 0 && b.hands as f64 / total >= min_frequency)
            .map(|(i, b)| TcRow {
                true_count: MIN_TC_BIN + i as i32,
                frequency: b.hands as f64 / total,
                advantage_at_tc: 100.0 * b.net / b.bet,
                avg_bet_at_tc: b.bet / b.hands as f64,
            })
            .collect()
    }

    /// Table d'avantage par bin pour la politique Kelly du modèle B.
    ///
    /// # Arguments
    ///
    /// * `min_hands` - Mains minimales pour qu'un bin soit fiable.
    ///
    /// # Returns
    ///
    /// La table, ou une erreur si les données sont insuffisantes.
    pub fn edge_table(&self, min_hands: u64) -> Result<EdgeTable, String> {
        let bins: Vec<(i32, u64, f64, f64)> = self
            .tc_bins
            .iter()
            .enumerate()
            .map(|(i, b)| (MIN_TC_BIN + i as i32, b.hands, b.net, b.bet))
            .collect();
        EdgeTable::from_bins(&bins, self.unit_variance(), min_hands)
    }
}

// ======================================================================
// Modèle B : courbes d'équité et drawdowns
// ======================================================================

/// Trajectoire échantillonnée d'une bankroll.
#[derive(Debug, Clone, PartialEq)]
pub struct BankrollTrace {
    /// Bankroll à chaque point d'échantillonnage (le premier est la valeur initiale).
    pub samples: Vec<f64>,
    /// Ruine à chaque point d'échantillonnage.
    pub ruined: Vec<bool>,
    /// Pire repli relatif du run (fraction).
    pub max_drawdown: f64,
}

/// Point agrégé d'une courbe d'équité.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct EquityPoint {
    /// Nombre de mains jouées au point d'échantillonnage.
    pub hand_index: u64,
    /// 5e percentile de la bankroll.
    pub p05: f64,
    /// 25e percentile.
    pub p25: f64,
    /// Médiane.
    pub p50: f64,
    /// 75e percentile.
    pub p75: f64,
    /// 95e percentile.
    pub p95: f64,
    /// Moyenne.
    pub mean: f64,
    /// Fraction des runs ruinés.
    pub frac_ruined: f64,
}

/// Percentile par interpolation linéaire (méthode « type 7 ») d'un tableau trié.
///
/// # Arguments
///
/// * `sorted` - Valeurs triées par ordre croissant (non vide).
/// * `q` - Quantile dans `[0, 1]`.
///
/// # Returns
///
/// La valeur interpolée.
pub fn percentile(sorted: &[f64], q: f64) -> f64 {
    assert!(!sorted.is_empty(), "percentile d'un tableau vide");
    assert!((0.0..=1.0).contains(&q), "quantile hors de [0, 1] : {q}");
    let pos = q * (sorted.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
}

/// Agrège les trajectoires de bankroll en courbe d'équité.
///
/// # Arguments
///
/// * `traces` - Trajectoires, toutes de même longueur.
/// * `interval` - Mains entre deux points d'échantillonnage.
///
/// # Returns
///
/// Un point par échantillon, ou une erreur si les trajectoires sont incohérentes.
pub fn summarize_equity(
    traces: &[BankrollTrace],
    interval: u64,
) -> Result<Vec<EquityPoint>, String> {
    let first = traces.first().ok_or("aucune trajectoire de bankroll")?;
    let n_samples = first.samples.len();
    if traces.iter().any(|t| t.samples.len() != n_samples) {
        return Err("trajectoires de longueurs différentes".to_string());
    }
    let n_runs = traces.len() as f64;
    let mut column = Vec::with_capacity(traces.len());
    let points = (0..n_samples)
        .map(|k| {
            column.clear();
            column.extend(traces.iter().map(|t| t.samples[k]));
            column.sort_by(f64::total_cmp);
            let ruined = traces.iter().filter(|t| t.ruined[k]).count() as f64;
            EquityPoint {
                hand_index: k as u64 * interval,
                p05: percentile(&column, 0.05),
                p25: percentile(&column, 0.25),
                p50: percentile(&column, 0.50),
                p75: percentile(&column, 0.75),
                p95: percentile(&column, 0.95),
                mean: column.iter().sum::<f64>() / n_runs,
                frac_ruined: ruined / n_runs,
            }
        })
        .collect();
    Ok(points)
}

/// Bin d'histogramme.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct HistogramBin {
    /// Borne basse incluse.
    pub bin_start: f64,
    /// Borne haute exclue (incluse pour le dernier bin).
    pub bin_end: f64,
    /// Nombre de valeurs.
    pub count: u64,
}

/// Histogramme à pas fixe sur `[0, 100]` (valeurs en %).
///
/// # Arguments
///
/// * `values_pct` - Valeurs en pourcentage.
/// * `bin_width` - Largeur d'un bin (diviseur de 100).
///
/// # Returns
///
/// Les bins de 0 à 100, y compris les bins vides.
pub fn histogram_pct(values_pct: &[f64], bin_width: f64) -> Vec<HistogramBin> {
    assert!(
        bin_width > 0.0 && bin_width <= 100.0,
        "largeur de bin invalide : {bin_width}"
    );
    let n_bins = (100.0 / bin_width).ceil() as usize;
    let mut counts = vec![0u64; n_bins];
    for &v in values_pct {
        let idx = ((v / bin_width).floor().max(0.0) as usize).min(n_bins - 1);
        counts[idx] += 1;
    }
    counts
        .into_iter()
        .enumerate()
        .map(|(i, count)| HistogramBin {
            bin_start: i as f64 * bin_width,
            bin_end: ((i + 1) as f64 * bin_width).min(100.0),
            count,
        })
        .collect()
}

#[cfg(test)]
mod tests;
