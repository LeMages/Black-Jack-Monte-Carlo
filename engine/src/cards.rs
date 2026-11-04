//! Cartes et sabot.
//!
//! Une carte est encodée par un `u8` dans `[1, 10]` : `1` = As, `2..=9` = valeur
//! nominale, `10` = toute carte valant 10 (10, J, Q, K). Le sabot contient
//! `n_decks` jeux, est mélangé par Fisher-Yates avec un PRNG injecté, et porte une
//! carte de coupe placée à `floor(total × penetration)`.

// Importation des crates externes
use rand::{Rng, RngExt};

/// Valeur encodée de l'As.
pub const ACE: u8 = 1;
/// Valeur encodée de toute carte à 10 (10, J, Q, K).
pub const TEN: u8 = 10;
/// Nombre de cartes dans un jeu standard.
pub const CARDS_PER_DECK: usize = 52;
/// Nombre minimal de cartes derrière la carte de coupe : garantit qu'une main
/// commencée avant la coupe peut toujours se terminer sans épuiser le sabot.
pub const MIN_CARDS_BEHIND_CUT: usize = 52;

/// Nombre d'exemplaires d'un rang dans un jeu de 52 cartes.
///
/// # Arguments
///
/// * `rank` - Rang encodé dans `[1, 10]`.
///
/// # Returns
///
/// 16 pour les cartes à 10, 4 pour les autres rangs.
pub fn copies_per_deck(rank: u8) -> u32 {
    debug_assert!((1..=10).contains(&rank), "rang invalide : {rank}");
    if rank == TEN {
        16
    } else {
        4
    }
}

/// Sabot de `n_decks` jeux avec carte de coupe.
///
/// Attributs principaux :
/// * `cards` - ordre courant des cartes ;
/// * `next` - index de la prochaine carte à tirer ;
/// * `cut_card` - index de la carte de coupe ;
/// * `remaining_by_rank` - cartes restantes par rang (index = rang).
#[derive(Debug, Clone)]
pub struct Shoe {
    cards: Vec<u8>,
    next: usize,
    cut_card: usize,
    remaining_by_rank: [u32; 11],
    n_decks: u8,
}

impl Shoe {
    /// Construit un sabot ordonné (non mélangé).
    ///
    /// # Arguments
    ///
    /// * `n_decks` - Nombre de jeux (≥ 1).
    /// * `penetration` - Fraction du sabot distribuée avant remélange, dans `]0, 1[`.
    ///
    /// # Returns
    ///
    /// Le sabot, à mélanger avec [`Shoe::shuffle`].
    ///
    /// # Panics
    ///
    /// Si les paramètres laissent moins de [`MIN_CARDS_BEHIND_CUT`] cartes derrière
    /// la coupe (invariant validé en amont par `Rules::validate`).
    pub fn new(n_decks: u8, penetration: f64) -> Self {
        assert!(n_decks >= 1, "n_decks doit être ≥ 1 (reçu {n_decks})");
        assert!(
            penetration > 0.0 && penetration < 1.0,
            "penetration doit être dans ]0, 1[ (reçu {penetration})"
        );
        let total = usize::from(n_decks) * CARDS_PER_DECK;
        let cut_card = cut_card_index(total, penetration);
        assert!(
            total - cut_card >= MIN_CARDS_BEHIND_CUT,
            "pénétration {penetration} trop profonde pour {n_decks} jeux : \
             {} cartes derrière la coupe, minimum {MIN_CARDS_BEHIND_CUT}",
            total - cut_card
        );
        let mut cards = Vec::with_capacity(total);
        for rank in 1..=10u8 {
            let copies = copies_per_deck(rank) * u32::from(n_decks);
            cards.extend(std::iter::repeat_n(rank, copies as usize));
        }
        let mut shoe = Self {
            cards,
            next: 0,
            cut_card,
            remaining_by_rank: [0; 11],
            n_decks,
        };
        shoe.refill_counts();
        shoe
    }

    /// Remet toutes les cartes dans le sabot et le mélange (Fisher-Yates).
    ///
    /// # Arguments
    ///
    /// * `rng` - Générateur injecté, seule source d'aléa du mélange.
    pub fn shuffle(&mut self, rng: &mut impl Rng) {
        for i in (1..self.cards.len()).rev() {
            let j = rng.random_range(0..=i);
            self.cards.swap(i, j);
        }
        self.next = 0;
        self.refill_counts();
    }

    /// Tire la prochaine carte.
    ///
    /// # Returns
    ///
    /// La carte encodée dans `[1, 10]`.
    ///
    /// # Panics
    ///
    /// Si le sabot est vide, ce que [`MIN_CARDS_BEHIND_CUT`] rend impossible tant
    /// que le remélange a lieu dès que la coupe est atteinte en fin de main.
    pub fn draw(&mut self) -> u8 {
        assert!(
            self.next < self.cards.len(),
            "sabot épuisé ({} cartes distribuées) : remélange manquant",
            self.next
        );
        let card = self.cards[self.next];
        self.next += 1;
        self.remaining_by_rank[usize::from(card)] -= 1;
        card
    }

    /// Indique si la carte de coupe a été atteinte (remélange en fin de main).
    pub fn needs_reshuffle(&self) -> bool {
        self.next >= self.cut_card
    }

    /// Nombre de cartes restant à distribuer.
    pub fn cards_remaining(&self) -> usize {
        self.cards.len() - self.next
    }

    /// Nombre exact de jeux restants (cartes restantes / 52).
    pub fn decks_remaining(&self) -> f64 {
        self.cards_remaining() as f64 / CARDS_PER_DECK as f64
    }

    /// Nombre de cartes restantes d'un rang donné.
    ///
    /// # Arguments
    ///
    /// * `rank` - Rang encodé dans `[1, 10]`.
    ///
    /// # Returns
    ///
    /// Le nombre d'exemplaires de ce rang non encore distribués.
    pub fn remaining_of(&self, rank: u8) -> u32 {
        assert!((1..=10).contains(&rank), "rang invalide : {rank}");
        self.remaining_by_rank[usize::from(rank)]
    }

    /// Nombre de jeux composant le sabot.
    pub fn n_decks(&self) -> u8 {
        self.n_decks
    }

    /// Nombre total de cartes du sabot.
    pub fn total_cards(&self) -> usize {
        self.cards.len()
    }

    fn refill_counts(&mut self) {
        self.remaining_by_rank = [0; 11];
        for rank in 1..=10u8 {
            self.remaining_by_rank[usize::from(rank)] =
                copies_per_deck(rank) * u32::from(self.n_decks);
        }
    }
}

/// Position de la carte de coupe : `floor(total × penetration)`.
///
/// # Arguments
///
/// * `total` - Nombre total de cartes du sabot.
/// * `penetration` - Fraction distribuée avant remélange.
///
/// # Returns
///
/// L'index de la carte de coupe.
pub fn cut_card_index(total: usize, penetration: f64) -> usize {
    (total as f64 * penetration).floor() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_xoshiro::Xoshiro256PlusPlus;

    #[test]
    fn shoe_has_expected_composition() {
        let shoe = Shoe::new(6, 0.75);
        assert_eq!(shoe.total_cards(), 312);
        assert_eq!(shoe.remaining_of(ACE), 24);
        assert_eq!(shoe.remaining_of(5), 24);
        assert_eq!(shoe.remaining_of(TEN), 96);
        assert!((shoe.decks_remaining() - 6.0).abs() < 1e-12);
    }

    #[test]
    fn shuffle_preserves_cards_and_is_deterministic() {
        let mut a = Shoe::new(2, 0.5);
        let mut b = Shoe::new(2, 0.5);
        a.shuffle(&mut Xoshiro256PlusPlus::seed_from_u64(7));
        b.shuffle(&mut Xoshiro256PlusPlus::seed_from_u64(7));
        assert_eq!(a.cards, b.cards);
        assert_ne!(a.cards, Shoe::new(2, 0.5).cards);
        let mut sorted = a.cards.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, Shoe::new(2, 0.5).cards);
    }

    #[test]
    fn draw_updates_counts_and_cut_card() {
        let mut shoe = Shoe::new(6, 0.75);
        shoe.shuffle(&mut Xoshiro256PlusPlus::seed_from_u64(1));
        let mut tens = 0;
        for _ in 0..234 {
            assert!(!shoe.needs_reshuffle());
            if shoe.draw() == TEN {
                tens += 1;
            }
        }
        // 234 = floor(312 × 0.75) : la coupe est atteinte exactement ici
        assert!(shoe.needs_reshuffle());
        assert_eq!(shoe.cards_remaining(), 78);
        assert_eq!(shoe.remaining_of(TEN), 96 - tens);
        assert!((shoe.decks_remaining() - 1.5).abs() < 1e-12);
    }

    #[test]
    #[should_panic(expected = "trop profonde")]
    fn rejects_too_deep_penetration() {
        let _shoe = Shoe::new(1, 0.9);
    }
}
