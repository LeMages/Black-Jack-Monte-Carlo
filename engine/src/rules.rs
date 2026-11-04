//! Règles de la table simulée.
//!
//! Les valeurs par défaut correspondent au jeu de référence du projet :
//! 6 jeux, S17, 3:2, DAS, double sur deux cartes quelconques, resplit jusqu'à
//! 4 mains, pas de resplit des As, une carte par As splitté, late surrender et
//! peek du croupier (règle US).

// Importation des crates externes
use serde::{Deserialize, Serialize};

// Importation des modules internes
use crate::cards::{cut_card_index, CARDS_PER_DECK, MIN_CARDS_BEHIND_CUT};

/// Règles de jeu paramétrables.
///
/// Attributs principaux :
/// * `n_decks` - nombre de jeux du sabot ;
/// * `penetration` - fraction distribuée avant remélange ;
/// * `dealer_hits_soft_17` - H17 si vrai, S17 sinon ;
/// * `blackjack_payout` - paiement du blackjack (1.5 = 3:2) ;
/// * `max_splits` - nombre maximal de splits par main initiale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Rules {
    /// Nombre de jeux du sabot.
    pub n_decks: u8,
    /// Fraction du sabot distribuée avant la carte de coupe.
    pub penetration: f64,
    /// Le croupier tire sur soft 17 (H17).
    pub dealer_hits_soft_17: bool,
    /// Multiplicateur de paiement du blackjack naturel.
    pub blackjack_payout: f64,
    /// Double autorisé après split (DAS).
    pub double_after_split: bool,
    /// Double autorisé sur deux cartes quelconques (sinon 9-11 seulement).
    pub double_any_two: bool,
    /// Nombre maximal de splits (3 = jusqu'à 4 mains).
    pub max_splits: u8,
    /// Resplit des As autorisé.
    pub resplit_aces: bool,
    /// Tirage autorisé sur des As splittés.
    pub hit_split_aces: bool,
    /// Abandon tardif (après peek) autorisé.
    pub late_surrender: bool,
    /// Le croupier vérifie le blackjack avec un As ou un 10 visible.
    pub dealer_peeks: bool,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            n_decks: 6,
            penetration: 0.75,
            dealer_hits_soft_17: false,
            blackjack_payout: 1.5,
            double_after_split: true,
            double_any_two: true,
            max_splits: 3,
            resplit_aces: false,
            hit_split_aces: false,
            late_surrender: true,
            dealer_peeks: true,
        }
    }
}

impl Rules {
    /// Vérifie la cohérence des règles.
    ///
    /// # Returns
    ///
    /// `Ok(())` si les règles sont simulables, sinon un message décrivant le
    /// champ invalide et la valeur reçue.
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=8).contains(&self.n_decks) {
            return Err(format!(
                "n_decks doit être dans [1, 8] (reçu {})",
                self.n_decks
            ));
        }
        if self.penetration.is_nan() || self.penetration <= 0.0 || self.penetration >= 1.0 {
            return Err(format!(
                "penetration doit être dans ]0, 1[ (reçu {})",
                self.penetration
            ));
        }
        let total = usize::from(self.n_decks) * CARDS_PER_DECK;
        let behind = total - cut_card_index(total, self.penetration);
        if behind < MIN_CARDS_BEHIND_CUT {
            return Err(format!(
                "penetration {} laisse {behind} cartes derrière la coupe (minimum {MIN_CARDS_BEHIND_CUT})",
                self.penetration
            ));
        }
        if self.blackjack_payout.is_nan() || self.blackjack_payout <= 0.0 {
            return Err(format!(
                "blackjack_payout doit être > 0 (reçu {})",
                self.blackjack_payout
            ));
        }
        if self.max_splits > 3 {
            return Err(format!(
                "max_splits doit être ≤ 3 (reçu {})",
                self.max_splits
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_reference_game() {
        let rules = Rules::default();
        assert_eq!(rules.n_decks, 6);
        assert!(!rules.dealer_hits_soft_17);
        assert_eq!(rules.max_splits, 3);
        assert!(rules.late_surrender && rules.double_after_split && rules.dealer_peeks);
        assert!(rules.validate().is_ok());
    }

    #[test]
    fn rejects_invalid_rules() {
        let deep = Rules {
            penetration: 0.95,
            ..Rules::default()
        };
        assert!(deep.validate().is_err());
        let splits = Rules {
            max_splits: 5,
            ..Rules::default()
        };
        assert!(splits.validate().is_err());
    }
}
