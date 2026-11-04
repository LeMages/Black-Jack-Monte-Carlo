//! Systèmes de comptage de cartes.
//!
//! Chaque système est décrit par une table de tags ([`CountSystem`]) et compté par
//! un unique [`TagCounter`] paramétré : aucune logique n'est dupliquée par système.
//!
//! * Systèmes équilibrés : IRC = 0, `true_count = running_count / jeux restants`.
//! * KO (non équilibré) : IRC = `4 × (1 − n_decks)`, mise pilotée par le running
//!   count brut comparé au pivot +4. Pour les déviations, le running count est
//!   converti en compte équilibré exact : `rc − IRC − 4 × jeux joués`, divisé par
//!   les jeux restants (KO = compte équilibré + 4 par jeu, le 7 en plus).
//! * Side-count des As (Hi-Opt II, Omega II) : l'As vaut 0 au jeu ; le compte de
//!   mise ajoute `ace_factor × (As restants − 4 × jeux restants) / jeux restants`.
//!   Le compte de décision n'inclut pas cette correction.
//!
//! Le true count est renvoyé non tronqué ; le moteur de jeu le tronque vers −∞
//! (`floor`) pour les décisions.

// Importation des crates externes
use serde::Serialize;

// Importation des modules internes
use crate::cards::{copies_per_deck, ACE};

/// Facteur par défaut de la correction du side-count des As (systèmes niveau 2).
pub const DEFAULT_ACE_FACTOR: f64 = 2.0;
/// Pivot du KO : la rampe de mise démarre quand le running count l'atteint.
pub const KO_PIVOT: f64 = 4.0;

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
    /// * `decks_remaining` - Jeux restants (valeur exacte, > 0).
    fn true_count(&self, decks_remaining: f64) -> f64;

    /// Compte qui pilote la mise.
    ///
    /// # Arguments
    ///
    /// * `decks_remaining` - Jeux restants (valeur exacte, > 0).
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

// ======================================================================
// Définition des systèmes
// ======================================================================

/// Description statique d'un système de comptage.
///
/// Attributs principaux :
/// * `id` - identifiant stable (slug utilisé dans les données exportées) ;
/// * `tags` - valeur de chaque rang, index 0 = As, 9 = dix ;
/// * `balanced` - somme des tags nulle sur un jeu complet ;
/// * `ace_side_count` - As suivis à part pour corriger le compte de mise ;
/// * `betting_correlation` / `playing_efficiency` / `insurance_correlation` -
///   BC, PE et IC publiés.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CountSystem {
    /// Identifiant stable (`hi-lo`, `ko`, …).
    pub id: &'static str,
    /// Nom d'affichage.
    pub name: &'static str,
    /// Tags par rang : index 0 = As, 1 = 2, …, 9 = dix.
    pub tags: [f64; 10],
    /// Système équilibré.
    pub balanced: bool,
    /// Niveau du système (valeur absolue maximale des tags, arrondie).
    pub level: u8,
    /// Side-count des As activé.
    pub ace_side_count: bool,
    /// Betting correlation publiée.
    pub betting_correlation: f64,
    /// Playing efficiency publiée.
    pub playing_efficiency: f64,
    /// Insurance correlation publiée.
    pub insurance_correlation: f64,
}

/// Hi-Lo (Harvey Dubner, 1963).
pub const HI_LO: CountSystem = CountSystem {
    id: "hi-lo",
    name: "Hi-Lo",
    tags: [-1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, -1.0],
    balanced: true,
    level: 1,
    ace_side_count: false,
    betting_correlation: 0.97,
    playing_efficiency: 0.51,
    insurance_correlation: 0.76,
};

/// KO / Knock-Out (Vancura & Fuchs), non équilibré.
pub const KO: CountSystem = CountSystem {
    id: "ko",
    name: "KO",
    tags: [-1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, -1.0],
    balanced: false,
    level: 1,
    ace_side_count: false,
    betting_correlation: 0.98,
    playing_efficiency: 0.55,
    insurance_correlation: 0.78,
};

/// Hi-Opt II, avec side-count des As.
pub const HI_OPT_II: CountSystem = CountSystem {
    id: "hi-opt-ii",
    name: "Hi-Opt II",
    tags: [0.0, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0, 0.0, 0.0, -2.0],
    balanced: true,
    level: 2,
    ace_side_count: true,
    betting_correlation: 0.91,
    playing_efficiency: 0.67,
    insurance_correlation: 0.91,
};

/// Omega II (Bryce Carlson), avec side-count des As.
pub const OMEGA_II: CountSystem = CountSystem {
    id: "omega-ii",
    name: "Omega II",
    tags: [0.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 0.0, -1.0, -2.0],
    balanced: true,
    level: 2,
    ace_side_count: true,
    betting_correlation: 0.92,
    playing_efficiency: 0.67,
    insurance_correlation: 0.85,
};

/// Wong Halves (Stanford Wong), tags fractionnaires.
pub const WONG_HALVES: CountSystem = CountSystem {
    id: "wong-halves",
    name: "Wong Halves",
    tags: [-1.0, 0.5, 1.0, 1.0, 1.5, 1.0, 0.5, 0.0, -0.5, -1.0],
    balanced: true,
    level: 3,
    ace_side_count: false,
    betting_correlation: 0.99,
    playing_efficiency: 0.56,
    insurance_correlation: 0.72,
};

/// Zen Count (Arnold Snyder).
pub const ZEN: CountSystem = CountSystem {
    id: "zen",
    name: "Zen Count",
    tags: [-1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 0.0, 0.0, -2.0],
    balanced: true,
    level: 2,
    ace_side_count: false,
    betting_correlation: 0.96,
    playing_efficiency: 0.63,
    insurance_correlation: 0.85,
};

/// Les six systèmes comparés, dans l'ordre d'affichage.
pub const ALL_SYSTEMS: [&CountSystem; 6] = [&HI_LO, &KO, &HI_OPT_II, &OMEGA_II, &WONG_HALVES, &ZEN];

/// Recherche un système par identifiant.
///
/// # Arguments
///
/// * `id` - Identifiant (`hi-lo`, `ko`, `hi-opt-ii`, `omega-ii`, `wong-halves`, `zen`).
///
/// # Returns
///
/// Le système, ou `None` si l'identifiant est inconnu.
pub fn system_by_id(id: &str) -> Option<&'static CountSystem> {
    ALL_SYSTEMS.iter().copied().find(|s| s.id == id)
}

impl CountSystem {
    /// Tag d'une carte.
    ///
    /// # Arguments
    ///
    /// * `card` - Carte encodée dans `[1, 10]`.
    ///
    /// # Returns
    ///
    /// La valeur ajoutée au running count.
    pub fn tag(&self, card: u8) -> f64 {
        self.tags[usize::from(card) - 1]
    }

    /// Somme des tags sur un jeu complet de 52 cartes (0 si équilibré, +4 pour KO).
    pub fn full_deck_sum(&self) -> f64 {
        (1..=10u8)
            .map(|rank| self.tag(rank) * f64::from(copies_per_deck(rank)))
            .sum()
    }

    /// Running count initial (IRC) pour un sabot de `n_decks` jeux.
    ///
    /// # Arguments
    ///
    /// * `n_decks` - Nombre de jeux.
    ///
    /// # Returns
    ///
    /// 0 pour un système équilibré, `−somme_par_jeu × (n_decks − 1)` sinon
    /// (−20 pour KO en 6 jeux).
    pub fn initial_running_count(&self, n_decks: u8) -> f64 {
        if self.balanced {
            0.0
        } else {
            -self.full_deck_sum() * (f64::from(n_decks) - 1.0)
        }
    }

    /// Écart-type des tags centrés sur un jeu complet.
    pub fn tag_std(&self) -> f64 {
        let mean = self.full_deck_sum() / 52.0;
        let var: f64 = (1..=10u8)
            .map(|rank| (self.tag(rank) - mean).powi(2) * f64::from(copies_per_deck(rank)))
            .sum::<f64>()
            / 52.0;
        var.sqrt()
    }

    /// Facteur d'échelle par rapport à Hi-Lo (ratio des écarts-types des tags).
    ///
    /// Sert à transposer les index de déviation Hi-Lo et la rampe de mise sur
    /// l'échelle du système (≈ 1,67 pour Hi-Opt II, ≈ 1,79 pour Zen).
    pub fn scale_vs_hi_lo(&self) -> f64 {
        self.tag_std() / HI_LO.tag_std()
    }
}

// ======================================================================
// Compteur à tags
// ======================================================================

/// Compteur générique piloté par un [`CountSystem`].
///
/// Attributs principaux :
/// * `running` - running count courant ;
/// * `aces_seen` - As observés depuis le remélange (side-count) ;
/// * `ace_factor` - poids de la correction du side-count dans le compte de mise.
#[derive(Debug, Clone)]
pub struct TagCounter {
    system: &'static CountSystem,
    running: f64,
    irc: f64,
    n_decks: u8,
    aces_seen: u32,
    ace_factor: f64,
}

impl TagCounter {
    /// Crée un compteur remis à l'IRC.
    ///
    /// # Arguments
    ///
    /// * `system` - Système de comptage.
    /// * `n_decks` - Nombre de jeux du sabot.
    /// * `ace_factor` - Poids du side-count des As (ignoré sans side-count).
    ///
    /// # Returns
    ///
    /// Le compteur prêt à observer des cartes.
    pub fn new(system: &'static CountSystem, n_decks: u8, ace_factor: f64) -> Self {
        let mut counter = Self {
            system,
            running: 0.0,
            irc: 0.0,
            n_decks,
            aces_seen: 0,
            ace_factor,
        };
        counter.reset(n_decks);
        counter
    }

    /// Système compté.
    pub fn system(&self) -> &'static CountSystem {
        self.system
    }

    /// Excédent d'As restants par rapport à l'attendu (`4 × jeux restants`).
    ///
    /// # Arguments
    ///
    /// * `decks_remaining` - Jeux restants (valeur exacte).
    ///
    /// # Returns
    ///
    /// Positif si le sabot restant est riche en As.
    pub fn aces_rich(&self, decks_remaining: f64) -> f64 {
        let remaining = f64::from(4 * u32::from(self.n_decks)) - f64::from(self.aces_seen);
        remaining - 4.0 * decks_remaining
    }
}

impl Counter for TagCounter {
    fn reset(&mut self, n_decks: u8) {
        self.n_decks = n_decks;
        self.irc = self.system.initial_running_count(n_decks);
        self.running = self.irc;
        self.aces_seen = 0;
    }

    fn observe(&mut self, card: u8) {
        self.running += self.system.tag(card);
        if card == ACE {
            self.aces_seen += 1;
        }
    }

    fn running_count(&self) -> f64 {
        self.running
    }

    fn true_count(&self, decks_remaining: f64) -> f64 {
        debug_assert!(decks_remaining > 0.0, "jeux restants nuls");
        if self.system.balanced {
            return self.running / decks_remaining;
        }
        // Conversion KO → compte équilibré : on retire l'IRC et le biais attendu
        // des cartes déjà jouées (somme par jeu × jeux joués)
        let decks_played = f64::from(self.n_decks) - decks_remaining;
        let balanced = self.running - self.irc - self.system.full_deck_sum() * decks_played;
        balanced / decks_remaining
    }

    fn betting_count(&self, decks_remaining: f64) -> f64 {
        debug_assert!(decks_remaining > 0.0, "jeux restants nuls");
        if !self.system.balanced {
            return self.running;
        }
        let correction = if self.system.ace_side_count {
            self.ace_factor * self.aces_rich(decks_remaining)
        } else {
            0.0
        };
        (self.running + correction) / decks_remaining
    }
}

#[cfg(test)]
mod tests;
