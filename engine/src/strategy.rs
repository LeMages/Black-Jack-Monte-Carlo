//! Basic strategy 4-8 jeux (S17 et H17, DAS, late surrender).
//!
//! Les tables reproduisent les charts de Wizard of Odds pour 4 à 8 jeux
//! (<https://wizardofodds.com/games/blackjack/strategy/4-decks/>), qui tiennent
//! compte du double après split et du late surrender. Chaque cellule encode une
//! action avec son repli lorsque l'action n'est pas permise :
//!
//! | Code | Sens |
//! |---|---|
//! | `H` / `S` | tirer / rester |
//! | `D` | doubler, sinon tirer (Dh) |
//! | `d` | doubler, sinon rester (Ds) |
//! | `R` | abandonner, sinon tirer (Rh) |
//! | `r` | abandonner, sinon rester (Rs) |
//! | `P` | splitter |
//! | `p` | splitter seulement si DAS (Ph) |
//! | `Q` | abandonner, sinon splitter (Rp) |
//! | `-` | pas de split : utiliser la table des totaux |
//!
//! Colonnes : carte visible du croupier 2, 3, 4, 5, 6, 7, 8, 9, 10, As.

// Importation des modules internes
use crate::cards::ACE;
use crate::rules::Rules;

/// Action finale du joueur, après application des replis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    /// Tirer une carte.
    Hit,
    /// Rester.
    Stand,
    /// Doubler la mise et recevoir une seule carte.
    Double,
    /// Séparer la paire en deux mains.
    Split,
    /// Abandonner la moitié de la mise.
    Surrender,
}

/// Situation de décision transmise à une stratégie de jeu.
///
/// Attributs principaux :
/// * `total` / `is_soft` - valeur de la main ;
/// * `pair_rank` - rang de la paire, si la main en est une ;
/// * `dealer_up` - carte visible du croupier ;
/// * `can_double` / `can_split` / `can_surrender` - actions permises ici ;
/// * `true_count` - true count de décision, déjà tronqué vers −∞.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecisionContext {
    /// Total de la main (As promu si possible).
    pub total: u8,
    /// Vrai si un As compte pour 11.
    pub is_soft: bool,
    /// Rang de la paire, `None` si la main n'est pas une paire.
    pub pair_rank: Option<u8>,
    /// Carte visible du croupier (1 = As).
    pub dealer_up: u8,
    /// Le double est permis dans cette situation.
    pub can_double: bool,
    /// Le split est permis dans cette situation.
    pub can_split: bool,
    /// L'abandon est permis dans cette situation.
    pub can_surrender: bool,
    /// True count de décision (tronqué vers −∞).
    pub true_count: f64,
}

/// Stratégie de jeu injectée dans le moteur.
pub trait PlayingStrategy {
    /// Choisit l'action à jouer.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Situation de décision.
    ///
    /// # Returns
    ///
    /// Une action permise dans `ctx`.
    fn decide(&self, ctx: &DecisionContext) -> PlayerAction;

    /// Décide de prendre l'assurance face à un As.
    ///
    /// # Arguments
    ///
    /// * `true_count` - True count de décision (tronqué).
    ///
    /// # Returns
    ///
    /// `true` pour assurer.
    fn take_insurance(&self, true_count: f64) -> bool;
}

// ======================================================================
// Tables
// ======================================================================

type Row = &'static [u8; 10];

/// Totaux durs 4 à 21, S17.
const HARD_S17: [Row; 18] = [
    b"HHHHHHHHHH", // 4
    b"HHHHHHHHHH", // 5
    b"HHHHHHHHHH", // 6
    b"HHHHHHHHHH", // 7
    b"HHHHHHHHHH", // 8
    b"HDDDDHHHHH", // 9
    b"DDDDDDDDHH", // 10
    b"DDDDDDDDDH", // 11
    b"HHSSSHHHHH", // 12
    b"SSSSSHHHHH", // 13
    b"SSSSSHHHHH", // 14
    b"SSSSSHHHRH", // 15
    b"SSSSSHHRRR", // 16
    b"SSSSSSSSSS", // 17
    b"SSSSSSSSSS", // 18
    b"SSSSSSSSSS", // 19
    b"SSSSSSSSSS", // 20
    b"SSSSSSSSSS", // 21
];

/// Totaux durs 4 à 21, H17 (11 v A double, abandons 15 v A et 17 v A).
const HARD_H17: [Row; 18] = [
    b"HHHHHHHHHH", // 4
    b"HHHHHHHHHH", // 5
    b"HHHHHHHHHH", // 6
    b"HHHHHHHHHH", // 7
    b"HHHHHHHHHH", // 8
    b"HDDDDHHHHH", // 9
    b"DDDDDDDDHH", // 10
    b"DDDDDDDDDD", // 11
    b"HHSSSHHHHH", // 12
    b"SSSSSHHHHH", // 13
    b"SSSSSHHHHH", // 14
    b"SSSSSHHHRR", // 15
    b"SSSSSHHRRR", // 16
    b"SSSSSSSSSr", // 17
    b"SSSSSSSSSS", // 18
    b"SSSSSSSSSS", // 19
    b"SSSSSSSSSS", // 20
    b"SSSSSSSSSS", // 21
];

/// Totaux souples 12 (A,A non splittable) à 21, S17.
const SOFT_S17: [Row; 10] = [
    b"HHHHHHHHHH", // 12
    b"HHHDDHHHHH", // 13 (A,2)
    b"HHHDDHHHHH", // 14 (A,3)
    b"HHDDDHHHHH", // 15 (A,4)
    b"HHDDDHHHHH", // 16 (A,5)
    b"HDDDDHHHHH", // 17 (A,6)
    b"SddddSSHHH", // 18 (A,7)
    b"SSSSSSSSSS", // 19 (A,8)
    b"SSSSSSSSSS", // 20
    b"SSSSSSSSSS", // 21
];

/// Totaux souples 12 à 21, H17 (A,7 v 2 et A,8 v 6 doublent).
const SOFT_H17: [Row; 10] = [
    b"HHHHHHHHHH", // 12
    b"HHHDDHHHHH", // 13
    b"HHHDDHHHHH", // 14
    b"HHDDDHHHHH", // 15
    b"HHDDDHHHHH", // 16
    b"HDDDDHHHHH", // 17
    b"dddddSSHHH", // 18
    b"SSSSdSSSSS", // 19
    b"SSSSSSSSSS", // 20
    b"SSSSSSSSSS", // 21
];

/// Paires par rang (index 0 = As, 9 = dix), S17.
const PAIRS_S17: [Row; 10] = [
    b"PPPPPPPPPP", // A,A
    b"ppPPPP----", // 2,2
    b"ppPPPP----", // 3,3
    b"---pp-----", // 4,4
    b"----------", // 5,5 (joué comme 10 dur)
    b"pPPPP-----", // 6,6
    b"PPPPPP----", // 7,7
    b"PPPPPPPPPP", // 8,8
    b"PPPPP-PP--", // 9,9
    b"----------", // 10,10
];

/// Paires par rang, H17 (8,8 v A : abandon sinon split).
const PAIRS_H17: [Row; 10] = [
    b"PPPPPPPPPP",
    b"ppPPPP----",
    b"ppPPPP----",
    b"---pp-----",
    b"----------",
    b"pPPPP-----",
    b"PPPPPP----",
    b"PPPPPPPPPQ",
    b"PPPPP-PP--",
    b"----------",
];

/// Index de colonne pour une carte visible du croupier.
///
/// # Arguments
///
/// * `dealer_up` - Carte visible (1 = As, 2..=10).
///
/// # Returns
///
/// 0 pour un 2, …, 8 pour un 10, 9 pour un As.
pub fn dealer_column(dealer_up: u8) -> usize {
    debug_assert!(
        (1..=10).contains(&dealer_up),
        "carte croupier invalide : {dealer_up}"
    );
    if dealer_up == ACE {
        9
    } else {
        usize::from(dealer_up) - 2
    }
}

// ======================================================================
// Stratégie de base
// ======================================================================

/// Basic strategy 4-8 jeux sélectionnée selon S17/H17 et DAS.
///
/// Attributs principaux :
/// * `hard` / `soft` / `pairs` - tables actives ;
/// * `das` - double après split autorisé (active les cellules `p`).
#[derive(Debug, Clone, Copy)]
pub struct BasicStrategy {
    hard: &'static [Row; 18],
    soft: &'static [Row; 10],
    pairs: &'static [Row; 10],
    das: bool,
}

impl BasicStrategy {
    /// Construit la basic strategy adaptée aux règles.
    ///
    /// # Arguments
    ///
    /// * `rules` - Règles de la table (S17/H17 et DAS sont utilisés).
    ///
    /// # Returns
    ///
    /// La stratégie prête à l'emploi.
    pub fn new(rules: &Rules) -> Self {
        if rules.dealer_hits_soft_17 {
            Self {
                hard: &HARD_H17,
                soft: &SOFT_H17,
                pairs: &PAIRS_H17,
                das: rules.double_after_split,
            }
        } else {
            Self {
                hard: &HARD_S17,
                soft: &SOFT_S17,
                pairs: &PAIRS_S17,
                das: rules.double_after_split,
            }
        }
    }

    /// Indique si la basic strategy abandonne dans cette situation.
    ///
    /// Pour une paire splittable, seule la table des paires décide (8,8 v 10
    /// se splitte en S17 au lieu d'être abandonné comme un 16 dur).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Situation de décision.
    ///
    /// # Returns
    ///
    /// `true` si la cellule demande l'abandon et que l'abandon est permis.
    pub fn wants_surrender(&self, ctx: &DecisionContext) -> bool {
        if !ctx.can_surrender {
            return false;
        }
        if let (Some(rank), true) = (ctx.pair_rank, ctx.can_split) {
            return self.pair_cell(rank, ctx.dealer_up) == b'Q';
        }
        matches!(self.total_cell(ctx), b'R' | b'r')
    }

    /// Indique si la basic strategy splitte cette paire.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Situation de décision.
    ///
    /// # Returns
    ///
    /// `true` si la main est une paire splittable et que la table le demande.
    pub fn wants_split(&self, ctx: &DecisionContext) -> bool {
        match (ctx.pair_rank, ctx.can_split) {
            (Some(rank), true) => match self.pair_cell(rank, ctx.dealer_up) {
                b'P' | b'Q' => true,
                b'p' => self.das,
                _ => false,
            },
            _ => false,
        }
    }

    /// Action tirée de la table des totaux, replis appliqués (ni split ni abandon).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Situation de décision.
    ///
    /// # Returns
    ///
    /// `Hit`, `Stand` ou `Double`.
    pub fn total_action(&self, ctx: &DecisionContext) -> PlayerAction {
        resolve_total_cell(self.total_cell(ctx), ctx.can_double)
    }

    fn total_cell(&self, ctx: &DecisionContext) -> u8 {
        let col = dealer_column(ctx.dealer_up);
        if ctx.is_soft {
            let row = usize::from(ctx.total.clamp(12, 21) - 12);
            self.soft[row][col]
        } else {
            let row = usize::from(ctx.total.clamp(4, 21) - 4);
            self.hard[row][col]
        }
    }

    fn pair_cell(&self, rank: u8, dealer_up: u8) -> u8 {
        self.pairs[usize::from(rank) - 1][dealer_column(dealer_up)]
    }
}

/// Convertit une cellule de la table des totaux en action, avec replis.
///
/// # Arguments
///
/// * `cell` - Code de cellule (`H`, `S`, `D`, `d`, `R`, `r`).
/// * `can_double` - Le double est permis.
///
/// # Returns
///
/// L'action finale hors split et abandon.
pub fn resolve_total_cell(cell: u8, can_double: bool) -> PlayerAction {
    match cell {
        b'D' if can_double => PlayerAction::Double,
        b'd' if can_double => PlayerAction::Double,
        b'S' | b'd' | b'r' => PlayerAction::Stand,
        _ => PlayerAction::Hit,
    }
}

impl PlayingStrategy for BasicStrategy {
    fn decide(&self, ctx: &DecisionContext) -> PlayerAction {
        if self.wants_surrender(ctx) {
            return PlayerAction::Surrender;
        }
        if self.wants_split(ctx) {
            return PlayerAction::Split;
        }
        self.total_action(ctx)
    }

    fn take_insurance(&self, _true_count: f64) -> bool {
        // La basic strategy ne prend jamais l'assurance (espérance négative)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hand::Hand;

    /// Contexte d'une main initiale à deux cartes (tout est permis).
    fn initial(cards: [u8; 2], dealer_up: u8) -> DecisionContext {
        let mut hand = Hand::new(false);
        hand.add(cards[0]);
        hand.add(cards[1]);
        let (total, is_soft) = hand.value();
        DecisionContext {
            total,
            is_soft,
            pair_rank: hand.pair_rank(),
            dealer_up,
            can_double: true,
            can_split: true,
            can_surrender: true,
            true_count: 0.0,
        }
    }

    fn s17() -> BasicStrategy {
        BasicStrategy::new(&Rules::default())
    }

    fn h17() -> BasicStrategy {
        BasicStrategy::new(&Rules {
            dealer_hits_soft_17: true,
            ..Rules::default()
        })
    }

    #[test]
    fn eleven_vs_ace_depends_on_soft_17_rule() {
        // Wizard of Odds 4-8 jeux : tirer en S17, doubler en H17
        assert_eq!(s17().decide(&initial([6, 5], ACE)), PlayerAction::Hit);
        assert_eq!(h17().decide(&initial([6, 5], ACE)), PlayerAction::Double);
        assert_eq!(s17().decide(&initial([6, 5], 10)), PlayerAction::Double);
    }

    #[test]
    fn sensitive_hard_cells() {
        let bs = s17();
        assert_eq!(bs.decide(&initial([10, 2], 3)), PlayerAction::Hit);
        assert_eq!(bs.decide(&initial([10, 2], 4)), PlayerAction::Stand);
        assert_eq!(bs.decide(&initial([10, 6], 10)), PlayerAction::Surrender);
        assert_eq!(bs.decide(&initial([10, 5], 10)), PlayerAction::Surrender);
        assert_eq!(bs.decide(&initial([10, 5], ACE)), PlayerAction::Hit);
        assert_eq!(
            h17().decide(&initial([10, 5], ACE)),
            PlayerAction::Surrender
        );
        assert_eq!(
            h17().decide(&initial([10, 7], ACE)),
            PlayerAction::Surrender
        );
        assert_eq!(bs.decide(&initial([5, 4], 2)), PlayerAction::Hit);
    }

    #[test]
    fn sensitive_soft_cells() {
        // A,7 v 2 : rester en S17 (WoO 4-8 jeux), doubler en H17
        assert_eq!(s17().decide(&initial([ACE, 7], 2)), PlayerAction::Stand);
        assert_eq!(h17().decide(&initial([ACE, 7], 2)), PlayerAction::Double);
        assert_eq!(s17().decide(&initial([ACE, 7], 3)), PlayerAction::Double);
        assert_eq!(s17().decide(&initial([ACE, 7], 9)), PlayerAction::Hit);
        assert_eq!(s17().decide(&initial([ACE, 8], 6)), PlayerAction::Stand);
        assert_eq!(h17().decide(&initial([ACE, 8], 6)), PlayerAction::Double);
    }

    #[test]
    fn sensitive_pair_cells() {
        let bs = s17();
        assert_eq!(bs.decide(&initial([9, 9], 7)), PlayerAction::Stand);
        assert_eq!(bs.decide(&initial([9, 9], 8)), PlayerAction::Split);
        for up in 1..=10 {
            assert_eq!(
                bs.decide(&initial([8, 8], up)),
                PlayerAction::Split,
                "8,8 v {up}"
            );
        }
        assert_eq!(h17().decide(&initial([8, 8], ACE)), PlayerAction::Surrender);
        assert_eq!(bs.decide(&initial([10, 10], 6)), PlayerAction::Stand);
        assert_eq!(bs.decide(&initial([5, 5], 9)), PlayerAction::Double);
        assert_eq!(bs.decide(&initial([ACE, ACE], ACE)), PlayerAction::Split);
    }

    #[test]
    fn das_only_splits_fall_back_without_das() {
        let no_das = BasicStrategy::new(&Rules {
            double_after_split: false,
            ..Rules::default()
        });
        assert_eq!(s17().decide(&initial([2, 2], 2)), PlayerAction::Split);
        assert_eq!(no_das.decide(&initial([2, 2], 2)), PlayerAction::Hit);
        assert_eq!(no_das.decide(&initial([4, 4], 5)), PlayerAction::Hit);
        assert_eq!(no_das.decide(&initial([6, 6], 3)), PlayerAction::Split);
    }

    #[test]
    fn impossible_actions_use_chart_fallbacks() {
        let bs = s17();
        let mut ctx = initial([ACE, 7], 3);
        ctx.can_double = false;
        assert_eq!(bs.decide(&ctx), PlayerAction::Stand); // Ds → stand
        let mut ctx = initial([6, 5], 10);
        ctx.can_double = false;
        assert_eq!(bs.decide(&ctx), PlayerAction::Hit); // Dh → hit
        let mut ctx = initial([10, 6], 10);
        ctx.can_surrender = false;
        assert_eq!(bs.decide(&ctx), PlayerAction::Hit); // Rh → hit
        let mut ctx = initial([10, 7], ACE);
        ctx.can_surrender = false;
        assert_eq!(h17().decide(&ctx), PlayerAction::Stand); // Rs → stand
        let mut ctx = initial([8, 8], ACE);
        ctx.can_surrender = false;
        assert_eq!(h17().decide(&ctx), PlayerAction::Split); // Rp → split
        let mut ctx = initial([8, 8], 10);
        ctx.can_split = false;
        assert_eq!(bs.decide(&ctx), PlayerAction::Surrender); // 16 dur
    }
}
