//! Valeur d'une main de blackjack.
//!
//! Deux représentations coexistent : la fonction pure [`hand_value`] sur une
//! tranche de cartes (référence, tests) et la structure incrémentale [`Hand`]
//! utilisée par le moteur de jeu, qui évite toute allocation par main.

// Importation des modules internes
use crate::cards::ACE;

/// Calcule la valeur d'une main.
///
/// Un As compte 1 ; un seul As peut être promu à 11 si le total ne dépasse pas 21
/// (en promouvoir deux dépasserait toujours 21).
///
/// # Arguments
///
/// * `cards` - Cartes encodées dans `[1, 10]`.
///
/// # Returns
///
/// `(total, is_soft)` où `is_soft` indique qu'un As compte pour 11.
pub fn hand_value(cards: &[u8]) -> (u8, bool) {
    let hard: u32 = cards.iter().map(|&c| u32::from(c)).sum();
    let has_ace = cards.contains(&ACE);
    soft_promotion(hard, has_ace)
}

/// Indique si une main est un blackjack : deux cartes totalisant 21, hors split.
///
/// # Arguments
///
/// * `cards` - Cartes de la main.
/// * `from_split` - Vrai si la main est issue d'un split.
///
/// # Returns
///
/// `true` pour un blackjack naturel.
pub fn is_blackjack(cards: &[u8], from_split: bool) -> bool {
    !from_split && cards.len() == 2 && hand_value(cards).0 == 21
}

/// Indique si une main dépasse 21.
///
/// # Arguments
///
/// * `cards` - Cartes de la main.
///
/// # Returns
///
/// `true` si le total est strictement supérieur à 21.
pub fn is_bust(cards: &[u8]) -> bool {
    hand_value(cards).0 > 21
}

/// Indique si une main est une paire (deux cartes de même valeur encodée).
///
/// # Arguments
///
/// * `cards` - Cartes de la main.
///
/// # Returns
///
/// `true` pour deux cartes de même rang (deux figures forment une paire de 10).
pub fn is_pair(cards: &[u8]) -> bool {
    cards.len() == 2 && cards[0] == cards[1]
}

fn soft_promotion(hard: u32, has_ace: bool) -> (u8, bool) {
    // Plafonné pour rester dans un u8 : au-delà de 21 seule la notion de bust compte
    let hard = hard.min(u32::from(u8::MAX)) as u8;
    if has_ace && hard + 10 <= 21 {
        (hard + 10, true)
    } else {
        (hard, false)
    }
}

/// Main incrémentale sans allocation.
///
/// Attributs principaux :
/// * `hard_total` - somme des cartes, As comptés 1 ;
/// * `n_cards` - nombre de cartes ;
/// * `first` / `second` - deux premières cartes (détection de paire) ;
/// * `from_split` - main issue d'un split (pas de blackjack possible).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Hand {
    hard_total: u8,
    has_ace: bool,
    n_cards: u8,
    first: u8,
    second: u8,
    from_split: bool,
}

impl Hand {
    /// Crée une main vide.
    ///
    /// # Arguments
    ///
    /// * `from_split` - Vrai si la main naît d'un split.
    ///
    /// # Returns
    ///
    /// Une main sans carte.
    pub fn new(from_split: bool) -> Self {
        Self {
            from_split,
            ..Self::default()
        }
    }

    /// Ajoute une carte à la main.
    ///
    /// # Arguments
    ///
    /// * `card` - Carte encodée dans `[1, 10]`.
    pub fn add(&mut self, card: u8) {
        debug_assert!((1..=10).contains(&card), "carte invalide : {card}");
        match self.n_cards {
            0 => self.first = card,
            1 => self.second = card,
            _ => {}
        }
        self.n_cards = self.n_cards.saturating_add(1);
        self.hard_total = self.hard_total.saturating_add(card);
        self.has_ace |= card == ACE;
    }

    /// Valeur courante `(total, is_soft)`, même règle que [`hand_value`].
    pub fn value(&self) -> (u8, bool) {
        soft_promotion(u32::from(self.hard_total), self.has_ace)
    }

    /// Total courant (As promu si possible).
    pub fn total(&self) -> u8 {
        self.value().0
    }

    /// Vrai si le total dépasse 21.
    pub fn is_bust(&self) -> bool {
        self.hard_total > 21
    }

    /// Vrai pour un blackjack naturel (2 cartes, 21, hors split).
    pub fn is_blackjack(&self) -> bool {
        !self.from_split && self.n_cards == 2 && self.total() == 21
    }

    /// Rang de la paire si la main est une paire de deux cartes.
    ///
    /// # Returns
    ///
    /// `Some(rank)` pour une paire, `None` sinon.
    pub fn pair_rank(&self) -> Option<u8> {
        (self.n_cards == 2 && self.first == self.second).then_some(self.first)
    }

    /// Nombre de cartes de la main.
    pub fn n_cards(&self) -> u8 {
        self.n_cards
    }

    /// Vrai si la main est issue d'un split.
    pub fn from_split(&self) -> bool {
        self.from_split
    }

    /// Première carte de la main (utilisée pour séparer une paire).
    pub fn first_card(&self) -> u8 {
        self.first
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_hard_and_soft_totals() {
        assert_eq!(hand_value(&[10, 7]), (17, false));
        assert_eq!(hand_value(&[1, 6]), (17, true));
        assert_eq!(hand_value(&[1, 6, 10]), (17, false));
        assert_eq!(hand_value(&[1, 1]), (12, true));
        assert_eq!(hand_value(&[1, 1, 9]), (21, true));
        assert_eq!(hand_value(&[1, 10]), (21, true));
        assert_eq!(hand_value(&[10, 10, 5]), (25, false));
    }

    #[test]
    fn detects_blackjack_bust_and_pair() {
        assert!(is_blackjack(&[1, 10], false));
        assert!(!is_blackjack(&[1, 10], true));
        assert!(!is_blackjack(&[7, 7, 7], false));
        assert!(is_bust(&[10, 6, 6]));
        assert!(!is_bust(&[1, 1, 10, 9]));
        assert!(is_pair(&[8, 8]));
        assert!(!is_pair(&[8, 8, 8]));
        assert!(!is_pair(&[9, 10]));
    }

    #[test]
    fn incremental_hand_matches_pure_function() {
        let sequences: [&[u8]; 5] = [
            &[1, 6],
            &[1, 6, 10],
            &[10, 10, 5],
            &[1, 1, 9],
            &[2, 3, 4, 1],
        ];
        for cards in sequences {
            let mut hand = Hand::new(false);
            for &c in cards {
                hand.add(c);
            }
            assert_eq!(hand.value(), hand_value(cards), "séquence {cards:?}");
            assert_eq!(hand.is_bust(), is_bust(cards));
            assert_eq!(hand.is_blackjack(), is_blackjack(cards, false));
        }
        let mut split = Hand::new(true);
        split.add(1);
        split.add(10);
        assert!(!split.is_blackjack());
        let mut pair = Hand::new(false);
        pair.add(8);
        pair.add(8);
        assert_eq!(pair.pair_rank(), Some(8));
    }
}
