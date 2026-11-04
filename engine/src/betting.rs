//! Modèles de pari.
//!
//! * **Modèle A (unités)** : bankroll infinie, mise en unités selon une rampe
//!   linéaire du compte. Sert aux métriques académiques (avantage, σ, N₀, RoR).
//! * **Modèle B (bankroll proportionnelle)** : bankroll finie dont la mise dépend
//!   d'une politique (pourcentage plat, proportionnelle au compte, Kelly
//!   fractionnaire). Sert aux courbes d'équité et aux drawdowns.

// Importation des crates externes
use serde::{Deserialize, Serialize};

// Importation des modules internes
use crate::counters::{CountSystem, KO_PIVOT};

/// Borne basse des bins de compte exportés.
pub const MIN_TC_BIN: i32 = -30;
/// Borne haute des bins de compte exportés.
pub const MAX_TC_BIN: i32 = 30;

/// Bin entier d'un compte : `floor(count)` borné à `[MIN_TC_BIN, MAX_TC_BIN]`.
///
/// # Arguments
///
/// * `count` - Compte de mise (true count, ou running count pour KO).
///
/// # Returns
///
/// Le bin entier.
pub fn tc_bin(count: f64) -> i32 {
    (count.floor() as i32).clamp(MIN_TC_BIN, MAX_TC_BIN)
}

// ======================================================================
// Modèle A : rampe en unités
// ======================================================================

/// Rampe de mise du modèle A.
///
/// Systèmes équilibrés : `bet = clamp(1, spread_max, floor(bc / échelle − ramp_offset))`,
/// où l'échelle ramène le compte du système sur l'échelle Hi-Lo afin que tous les
/// systèmes suivent la même rampe. KO : la rampe démarre au pivot, qui remplace
/// `ramp_offset` : `bet = clamp(1, spread_max, floor(rc − pivot))`, sans division
/// par les jeux restants.
///
/// Attributs principaux :
/// * `spread_max` - mise maximale en unités ;
/// * `ramp_offset` - décalage de la rampe (en true count Hi-Lo) ;
/// * `scale` - facteur d'échelle du système vs Hi-Lo ;
/// * `ko_pivot` - pivot si le système est non équilibré.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitRamp {
    spread_max: f64,
    ramp_offset: f64,
    scale: f64,
    ko_pivot: Option<f64>,
}

impl UnitRamp {
    /// Construit la rampe d'un système.
    ///
    /// # Arguments
    ///
    /// * `system` - Système de comptage.
    /// * `spread_max` - Mise maximale en unités (≥ 1).
    /// * `ramp_offset` - Décalage de la rampe.
    ///
    /// # Returns
    ///
    /// La rampe prête à l'emploi.
    pub fn new(system: &CountSystem, spread_max: f64, ramp_offset: f64) -> Self {
        assert!(
            spread_max >= 1.0,
            "spread_max doit être ≥ 1 (reçu {spread_max})"
        );
        Self {
            spread_max,
            ramp_offset,
            scale: system.scale_vs_hi_lo(),
            ko_pivot: (!system.balanced).then_some(KO_PIVOT),
        }
    }

    /// Rampe à mise plate (1 unité), pour la basic strategy seule.
    pub fn flat() -> Self {
        Self {
            spread_max: 1.0,
            ramp_offset: 0.0,
            scale: 1.0,
            ko_pivot: None,
        }
    }

    /// Mise en unités pour un compte de mise.
    ///
    /// # Arguments
    ///
    /// * `betting_count` - Compte de mise du système.
    ///
    /// # Returns
    ///
    /// La mise dans `[1, spread_max]`.
    pub fn units(&self, betting_count: f64) -> f64 {
        let raw = match self.ko_pivot {
            Some(pivot) => (betting_count - pivot).floor(),
            None => (betting_count / self.scale - self.ramp_offset).floor(),
        };
        raw.clamp(1.0, self.spread_max)
    }
}

// ======================================================================
// Métriques du modèle A
// ======================================================================

/// Nombre de mains N₀ pour que l'espérance égale un écart-type : `(σ / EV)²`.
///
/// # Arguments
///
/// * `ev_per_hand` - Espérance par main (unités).
/// * `std_per_hand` - Écart-type par main (unités).
///
/// # Returns
///
/// N₀, ou `f64::INFINITY` si l'espérance est nulle ou négative.
pub fn n0(ev_per_hand: f64, std_per_hand: f64) -> f64 {
    if ev_per_hand <= 0.0 {
        return f64::INFINITY;
    }
    (std_per_hand / ev_per_hand).powi(2)
}

/// Risque de ruine (approximation exponentielle) : `exp(−2 · EV · B / σ²)`.
///
/// # Arguments
///
/// * `ev_per_hand` - Espérance par main (unités).
/// * `std_per_hand` - Écart-type par main (unités).
/// * `bankroll_units` - Bankroll de référence (unités).
///
/// # Returns
///
/// Probabilité de ruine dans `[0, 1]` (1 si l'espérance est ≤ 0).
pub fn risk_of_ruin(ev_per_hand: f64, std_per_hand: f64, bankroll_units: f64) -> f64 {
    if ev_per_hand <= 0.0 || std_per_hand <= 0.0 {
        return 1.0;
    }
    (-2.0 * ev_per_hand * bankroll_units / std_per_hand.powi(2)).exp()
}

/// Fraction de Kelly : taille optimale de l'unité en fraction de bankroll, `EV / σ²`.
///
/// # Arguments
///
/// * `ev_per_hand` - Espérance par main (unités).
/// * `std_per_hand` - Écart-type par main (unités).
///
/// # Returns
///
/// La fraction, 0 si l'espérance est ≤ 0.
pub fn kelly_fraction(ev_per_hand: f64, std_per_hand: f64) -> f64 {
    if ev_per_hand <= 0.0 || std_per_hand <= 0.0 {
        return 0.0;
    }
    ev_per_hand / std_per_hand.powi(2)
}

// ======================================================================
// Modèle B : bankroll finie
// ======================================================================

/// Politique de mise du modèle B.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BankrollPolicy {
    /// `pct × B_t`, indépendamment du compte.
    FlatPct {
        /// Fraction de la bankroll misée à chaque main.
        pct: f64,
    },
    /// `unités de la rampe × unit_pct × B_t`.
    CountProportional {
        /// Valeur d'une unité en fraction de la bankroll.
        unit_pct: f64,
    },
    /// `k × max(0, edge(TC)) / variance × B_t`.
    FractionalKelly {
        /// Fraction de Kelly (0,25 ; 0,5 ; 1).
        k: f64,
    },
}

/// Paramètres de la bankroll du modèle B.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BankrollSpec {
    /// Politique de mise.
    pub policy: BankrollPolicy,
    /// Bankroll initiale (unités).
    #[serde(default = "default_initial_bankroll")]
    pub initial_bankroll: f64,
    /// Mise minimale de la table ; en dessous, le run est ruiné.
    #[serde(default = "default_min_bet")]
    pub min_bet: f64,
    /// Mise maximale de la table.
    #[serde(default = "default_max_bet")]
    pub max_bet: f64,
}

fn default_initial_bankroll() -> f64 {
    1000.0
}

fn default_min_bet() -> f64 {
    1.0
}

fn default_max_bet() -> f64 {
    100.0
}

impl BankrollSpec {
    /// Vérifie la cohérence des paramètres.
    ///
    /// # Returns
    ///
    /// `Ok(())` ou un message décrivant le paramètre invalide.
    pub fn validate(&self) -> Result<(), String> {
        if self.min_bet <= 0.0 || self.max_bet < self.min_bet {
            return Err(format!(
                "mises invalides : min_bet {} / max_bet {}",
                self.min_bet, self.max_bet
            ));
        }
        if self.initial_bankroll < self.min_bet {
            return Err(format!(
                "initial_bankroll {} inférieure à min_bet {}",
                self.initial_bankroll, self.min_bet
            ));
        }
        let fraction = match self.policy {
            BankrollPolicy::FlatPct { pct } => pct,
            BankrollPolicy::CountProportional { unit_pct } => unit_pct,
            BankrollPolicy::FractionalKelly { k } => k,
        };
        if !(fraction > 0.0 && fraction <= 1.0) {
            return Err(format!("fraction de politique hors de ]0, 1] : {fraction}"));
        }
        Ok(())
    }
}

/// Avantage mesuré par bin de compte, utilisé par la politique Kelly.
///
/// Attributs principaux :
/// * `edges` - avantage par unité misée, pour chaque bin de `MIN_TC_BIN` à `MAX_TC_BIN` ;
/// * `variance` - variance du résultat par unité misée.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeTable {
    edges: Vec<f64>,
    variance: f64,
}

impl EdgeTable {
    /// Construit la table à partir des sommes par bin du modèle A.
    ///
    /// Les bins trop peu observés reprennent l'avantage du bin fiable le plus
    /// proche, pour éviter qu'un bruit d'échantillonnage pilote une grosse mise.
    ///
    /// # Arguments
    ///
    /// * `bins` - `(bin, mains, gain net, mise)` pour chaque bin observé.
    /// * `variance` - Variance par unité misée (> 0).
    /// * `min_hands` - Mains minimales pour qu'un bin soit fiable.
    ///
    /// # Returns
    ///
    /// La table, ou une erreur si aucun bin n'est fiable.
    pub fn from_bins(
        bins: &[(i32, u64, f64, f64)],
        variance: f64,
        min_hands: u64,
    ) -> Result<Self, String> {
        if variance <= 0.0 {
            return Err(format!("variance par unité invalide : {variance}"));
        }
        let reliable: Vec<(i32, f64)> = bins
            .iter()
            .filter(|(_, hands, _, wagered)| *hands >= min_hands && *wagered > 0.0)
            .map(|&(bin, _, net, wagered)| (bin, net / wagered))
            .collect();
        if reliable.is_empty() {
            return Err(format!(
                "aucun bin de compte avec au moins {min_hands} mains"
            ));
        }
        let edges = (MIN_TC_BIN..=MAX_TC_BIN)
            .map(|bin| nearest_edge(&reliable, bin))
            .collect();
        Ok(Self { edges, variance })
    }

    /// Avantage estimé pour un compte de mise.
    ///
    /// # Arguments
    ///
    /// * `betting_count` - Compte de mise.
    ///
    /// # Returns
    ///
    /// L'avantage par unité misée du bin correspondant.
    pub fn edge(&self, betting_count: f64) -> f64 {
        self.edges[(tc_bin(betting_count) - MIN_TC_BIN) as usize]
    }

    /// Variance par unité misée.
    pub fn variance(&self) -> f64 {
        self.variance
    }
}

fn nearest_edge(reliable: &[(i32, f64)], bin: i32) -> f64 {
    reliable
        .iter()
        .min_by_key(|(b, _)| (b - bin).abs())
        .map(|&(_, edge)| edge)
        .unwrap_or(0.0)
}

/// État d'une bankroll du modèle B au fil des mains.
///
/// Attributs principaux :
/// * `bankroll` - valeur courante, figée en cas de ruine ;
/// * `peak` / `max_drawdown` - plus haut atteint et pire repli relatif ;
/// * `ruined` - vrai dès que la bankroll passe sous la mise minimale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BankrollState {
    /// Valeur courante de la bankroll.
    pub bankroll: f64,
    /// Plus haut historique.
    pub peak: f64,
    /// Pire repli depuis un plus haut, en fraction de ce plus haut.
    pub max_drawdown: f64,
    /// Run ruiné (bankroll figée).
    pub ruined: bool,
}

impl BankrollState {
    /// Bankroll initiale.
    ///
    /// # Arguments
    ///
    /// * `initial` - Valeur de départ (> 0).
    ///
    /// # Returns
    ///
    /// L'état initial.
    pub fn new(initial: f64) -> Self {
        Self {
            bankroll: initial,
            peak: initial,
            max_drawdown: 0.0,
            ruined: false,
        }
    }

    /// Mise de la prochaine main selon la politique.
    ///
    /// # Arguments
    ///
    /// * `spec` - Paramètres de bankroll.
    /// * `ramp` - Rampe du modèle A (politique proportionnelle au compte).
    /// * `edges` - Table d'avantage (politique Kelly, sinon ignorée).
    /// * `betting_count` - Compte de mise courant.
    ///
    /// # Returns
    ///
    /// La mise, ou `None` si le run est ruiné.
    pub fn next_bet(
        &self,
        spec: &BankrollSpec,
        ramp: &UnitRamp,
        edges: Option<&EdgeTable>,
        betting_count: f64,
    ) -> Option<f64> {
        if self.ruined {
            return None;
        }
        let raw = match spec.policy {
            BankrollPolicy::FlatPct { pct } => pct * self.bankroll,
            BankrollPolicy::CountProportional { unit_pct } => {
                ramp.units(betting_count) * unit_pct * self.bankroll
            }
            BankrollPolicy::FractionalKelly { k } => {
                let table = edges.expect("politique Kelly sans table d'avantage");
                k * table.edge(betting_count).max(0.0) / table.variance() * self.bankroll
            }
        };
        // Assis à la table, on mise au moins le minimum ; jamais plus que la bankroll
        Some(raw.clamp(spec.min_bet, spec.max_bet).min(self.bankroll))
    }

    /// Applique le résultat d'une main.
    ///
    /// # Arguments
    ///
    /// * `net` - Gain net de la main.
    /// * `min_bet` - Mise minimale : en dessous, le run est déclaré ruiné.
    pub fn settle(&mut self, net: f64, min_bet: f64) {
        debug_assert!(!self.ruined, "résultat appliqué à un run ruiné");
        // Plancher à 0 : sans fonds, le joueur ne peut pas doubler ni splitter, il ne
        // perd donc jamais plus que sa bankroll (approximation documentée)
        self.bankroll = (self.bankroll + net).max(0.0);
        if self.bankroll > self.peak {
            self.peak = self.bankroll;
        }
        let drawdown = (self.peak - self.bankroll) / self.peak;
        if drawdown > self.max_drawdown {
            self.max_drawdown = drawdown.min(1.0);
        }
        if self.bankroll < min_bet {
            self.ruined = true;
        }
    }
}

#[cfg(test)]
mod tests;
