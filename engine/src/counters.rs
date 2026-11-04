//! Systèmes de comptage de cartes.

/// Compteur de cartes observant les cartes visibles.
pub trait Counter {
    /// Remet le running count à l'IRC (remélange).
    ///
    /// # Arguments
    ///
    /// * `n_decks` - Nombre de jeux du sabot.
    fn reset(&mut self, n_decks: u8);

    /// Observe une carte rendue visible.
    ///
    /// # Arguments
    ///
    /// * `card` - Carte encodée dans `[1, 10]`.
    fn observe(&mut self, card: u8);

    /// Running count courant.
    fn running_count(&self) -> f64;

    /// True count utilisé pour les décisions de jeu (non tronqué).
    ///
    /// # Arguments
    ///
    /// * `decks_remaining` - Jeux restants (valeur exacte).
    fn true_count(&self, decks_remaining: f64) -> f64;

    /// Compte qui pilote la mise.
    ///
    /// # Arguments
    ///
    /// * `decks_remaining` - Jeux restants (valeur exacte).
    fn betting_count(&self, decks_remaining: f64) -> f64;
}

/// Compteur inerte, utilisé par la basic strategy seule.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullCounter;

impl Counter for NullCounter {
    fn reset(&mut self, _n_decks: u8) {}
    fn observe(&mut self, _card: u8) {}
    fn running_count(&self) -> f64 {
        0.0
    }
    fn true_count(&self, _decks_remaining: f64) -> f64 {
        0.0
    }
    fn betting_count(&self, _decks_remaining: f64) -> f64 {
        0.0
    }
}
