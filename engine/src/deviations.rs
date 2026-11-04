//! Déviations de stratégie selon le compte : Illustrious 18 et Fab 4.
//!
//! Index Hi-Lo pour 6 jeux S17 (*Blackjack Attack*, D. Schlesinger). Chaque règle
//! encode explicitement son sens :
//! * [`Direction::AtOrAbove`] : la déviation est jouée si `TC ≥ index` ;
//! * [`Direction::Below`] : la déviation est jouée si `TC < index`
//!   (ex. 13 v 2 : rester par défaut, tirer si TC < −1).
//!
//! Pour les autres systèmes, les index sont multipliés par
//! [`CountSystem::scale_vs_hi_lo`] (ratio des écarts-types des tags) puis
//! arrondis à l'entier le plus proche. C'est une approximation documentée : les
//! index publiés propres à chaque système peuvent différer d'une unité.

// Importation des crates externes
use serde::{Deserialize, Serialize};

// Importation des modules internes
use crate::cards::{ACE, TEN};
use crate::counters::CountSystem;
use crate::strategy::{BasicStrategy, DecisionContext, PlayerAction, PlayingStrategy};

/// Niveau de déviations activé.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviationLevel {
    /// Basic strategy pure.
    None,
    /// Assurance seule (TC ≥ +3).
    Insurance,
    /// Illustrious 18 (assurance comprise).
    I18,
    /// Illustrious 18 et Fab 4 (abandons).
    I18Fab4,
}

/// Sens d'application d'une déviation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Déviation si `TC ≥ index`.
    AtOrAbove,
    /// Déviation si `TC < index`.
    Below,
}

impl Direction {
    /// Indique si la déviation s'applique.
    ///
    /// # Arguments
    ///
    /// * `true_count` - True count de décision (tronqué).
    /// * `index` - Index de la règle.
    ///
    /// # Returns
    ///
    /// `true` si le compte déclenche la déviation.
    pub fn triggers(self, true_count: f64, index: f64) -> bool {
        match self {
            Direction::AtOrAbove => true_count >= index,
            Direction::Below => true_count < index,
        }
    }
}

/// Déviation de jeu sur un total dur (ou une paire de 10).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayDeviation {
    /// Total dur du joueur (20 pour la paire de 10).
    pub total: u8,
    /// Carte visible du croupier.
    pub dealer_up: u8,
    /// Index Hi-Lo.
    pub index: f64,
    /// Sens de la règle.
    pub direction: Direction,
    /// Action jouée quand la règle se déclenche.
    pub action: PlayerAction,
}

/// Abandon conditionnel (Fab 4) : abandonner si `TC ≥ index`, sinon jouer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurrenderDeviation {
    /// Total dur du joueur.
    pub total: u8,
    /// Carte visible du croupier.
    pub dealer_up: u8,
    /// Index Hi-Lo.
    pub index: f64,
}

/// Index Hi-Lo de l'assurance.
pub const INSURANCE_INDEX: f64 = 3.0;

const fn play(
    total: u8,
    dealer_up: u8,
    index: f64,
    direction: Direction,
    action: PlayerAction,
) -> PlayDeviation {
    PlayDeviation {
        total,
        dealer_up,
        index,
        direction,
        action,
    }
}

use Direction::{AtOrAbove, Below};
use PlayerAction::{Double, Hit, Split, Stand};

/// Illustrious 18 hors assurance (17 règles de jeu).
pub const ILLUSTRIOUS_18: [PlayDeviation; 17] = [
    play(16, TEN, 0.0, AtOrAbove, Stand),
    play(15, TEN, 4.0, AtOrAbove, Stand),
    play(20, 5, 5.0, AtOrAbove, Split),
    play(20, 6, 4.0, AtOrAbove, Split),
    play(10, TEN, 4.0, AtOrAbove, Double),
    play(12, 3, 2.0, AtOrAbove, Stand),
    play(12, 2, 3.0, AtOrAbove, Stand),
    play(11, ACE, 1.0, AtOrAbove, Double),
    play(9, 2, 1.0, AtOrAbove, Double),
    play(10, ACE, 4.0, AtOrAbove, Double),
    play(9, 7, 3.0, AtOrAbove, Double),
    play(16, 9, 5.0, AtOrAbove, Stand),
    play(13, 2, -1.0, Below, Hit),
    play(12, 4, 0.0, Below, Hit),
    play(12, 5, -2.0, Below, Hit),
    play(12, 6, -1.0, Below, Hit),
    play(13, 3, -2.0, Below, Hit),
];

/// Fab 4 : abandons conditionnels.
pub const FAB_4: [SurrenderDeviation; 4] = [
    SurrenderDeviation {
        total: 14,
        dealer_up: TEN,
        index: 3.0,
    },
    SurrenderDeviation {
        total: 15,
        dealer_up: TEN,
        index: 0.0,
    },
    SurrenderDeviation {
        total: 15,
        dealer_up: 9,
        index: 2.0,
    },
    SurrenderDeviation {
        total: 15,
        dealer_up: ACE,
        index: 1.0,
    },
];

/// Transpose un index Hi-Lo sur l'échelle d'un système (arrondi à l'entier).
///
/// # Arguments
///
/// * `index` - Index Hi-Lo.
/// * `scale` - Facteur d'échelle du système (1 pour Hi-Lo).
///
/// # Returns
///
/// L'index arrondi à l'entier le plus proche.
pub fn scale_index(index: f64, scale: f64) -> f64 {
    (index * scale).round()
}

/// Basic strategy surchargée par les déviations d'un niveau donné.
///
/// Attributs principaux :
/// * `basic` - stratégie de repli ;
/// * `plays` / `surrenders` - règles actives, index déjà mis à l'échelle ;
/// * `insurance_index` - index d'assurance, `None` si l'assurance est refusée.
#[derive(Debug, Clone)]
pub struct DeviationStrategy {
    basic: BasicStrategy,
    plays: Vec<PlayDeviation>,
    surrenders: Vec<SurrenderDeviation>,
    insurance_index: Option<f64>,
}

impl DeviationStrategy {
    /// Construit la stratégie pour un niveau et un système.
    ///
    /// # Arguments
    ///
    /// * `basic` - Basic strategy adaptée aux règles.
    /// * `level` - Niveau de déviations.
    /// * `system` - Système de comptage (pour l'échelle des index).
    ///
    /// # Returns
    ///
    /// La stratégie prête à l'emploi.
    pub fn new(basic: BasicStrategy, level: DeviationLevel, system: &CountSystem) -> Self {
        let scale = system.scale_vs_hi_lo();
        let plays = if level >= DeviationLevel::I18 {
            ILLUSTRIOUS_18
                .iter()
                .map(|d| PlayDeviation {
                    index: scale_index(d.index, scale),
                    ..*d
                })
                .collect()
        } else {
            Vec::new()
        };
        let surrenders = if level >= DeviationLevel::I18Fab4 {
            FAB_4
                .iter()
                .map(|d| SurrenderDeviation {
                    index: scale_index(d.index, scale),
                    ..*d
                })
                .collect()
        } else {
            Vec::new()
        };
        let insurance_index =
            (level >= DeviationLevel::Insurance).then(|| scale_index(INSURANCE_INDEX, scale));
        Self {
            basic,
            plays,
            surrenders,
            insurance_index,
        }
    }

    /// Décision d'abandon : Fab 4 prioritaire, sinon basic strategy.
    fn surrender(&self, ctx: &DecisionContext, basic_splits: bool) -> bool {
        if !ctx.can_surrender {
            return false;
        }
        let fab4 = self
            .surrenders
            .iter()
            .find(|d| !ctx.is_soft && d.total == ctx.total && d.dealer_up == ctx.dealer_up);
        match fab4 {
            Some(rule) if !basic_splits => ctx.true_count >= rule.index,
            _ => self.basic.wants_surrender(ctx),
        }
    }

    /// Règle de jeu applicable à cette situation, si elle se déclenche.
    fn triggered_play(&self, ctx: &DecisionContext) -> Option<PlayerAction> {
        if ctx.is_soft {
            return None;
        }
        let is_ten_pair = ctx.pair_rank == Some(TEN);
        self.plays
            .iter()
            .filter(|d| d.total == ctx.total && d.dealer_up == ctx.dealer_up)
            // Les règles de split ne visent que la paire de 10 splittable
            .filter(|d| d.action != Split || (is_ten_pair && ctx.can_split))
            .find(|d| d.direction.triggers(ctx.true_count, d.index))
            .map(|d| d.action)
    }
}

impl PlayingStrategy for DeviationStrategy {
    fn decide(&self, ctx: &DecisionContext) -> PlayerAction {
        // ------------------------------------------------------------------
        // 1. Abandon (Fab 4 puis basic strategy), avant tout split
        // ------------------------------------------------------------------
        let basic_splits = self.basic.wants_split(ctx);
        if self.surrender(ctx, basic_splits) {
            return PlayerAction::Surrender;
        }
        if basic_splits {
            return PlayerAction::Split;
        }
        // ------------------------------------------------------------------
        // 2. Déviations de jeu, avec le repli du chart si le double est interdit
        // ------------------------------------------------------------------
        match self.triggered_play(ctx) {
            Some(PlayerAction::Double) if !ctx.can_double => PlayerAction::Hit,
            Some(action) => action,
            None => self.basic.total_action(ctx),
        }
    }

    fn take_insurance(&self, true_count: f64) -> bool {
        self.insurance_index.is_some_and(|idx| true_count >= idx)
    }
}

#[cfg(test)]
mod tests;
