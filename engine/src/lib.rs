//! Moteur de simulation Monte Carlo du blackjack et des systèmes de comptage.
//!
//! La bibliothèque ne connaît ni le CLI ni le dashboard : elle simule, agrège et
//! expose des structures de résultats sérialisables.
//!
//! Modules, du plus bas niveau au plus haut :
//! [`cards`] → [`hand`] → [`rules`] → [`strategy`] → [`counters`] →
//! [`deviations`] → [`game`] → [`betting`] → [`aggregate`] → [`simulation`].

pub mod aggregate;
pub mod betting;
pub mod cards;
pub mod counters;
pub mod deviations;
pub mod game;
pub mod hand;
pub mod rules;
pub mod simulation;
pub mod strategy;
